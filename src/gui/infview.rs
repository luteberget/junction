use const_cstr::*;
use matches::*;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;

use crate::util;
use crate::app::App;
use crate::document::*;
use crate::document::infview::*;
use crate::document::view::*;
use crate::document::interlocking::*;
use crate::document::model::*;
use crate::document::objects::*;
use crate::gui::widgets;
use crate::gui::widgets::Draw;
use crate::gui::menus;
use crate::gui::draw_inf;
use crate::config::RailUIColorName;


pub fn inf_view(app :&mut App) {
    inf_toolbar(app);

    let size = unsafe { igGetContentRegionAvail_nonUDT2().into() };
    widgets::canvas(size,
                    app.config.color_u32(RailUIColorName::CanvasBackground),
                    const_cstr!("railwaycanvas").as_ptr(),
                    app.document.inf_view.view.clone(),
                    |draw| {
        scroll(&mut app.document.inf_view);
        let mut preview_route = None;
        context_menu(&mut app.document, draw, &mut preview_route);
        interact(app, draw);
        draw_inf(app, draw, preview_route);
        Some(())
    });
}

fn draw_inf(app :&mut App, draw :&Draw, preview_route :Option<usize>) {
    draw_inf::base(app,draw);
    if let Some(r) = preview_route { draw_inf::route(app, draw, r); }
    draw_inf::state(app,draw);
    draw_inf::trains(app,draw);
}

fn scroll(inf_view :&mut InfView) { 
    unsafe {
        //if !igIsWindowFocused(0 as _) { return; }
        if !igIsItemHovered(0){ return; }
        let io = igGetIO();
        let wheel = (*io).MouseWheel;
        if wheel != 0.0 {
            inf_view.view.zoom(wheel);
        }
        if ((*io).KeyCtrl && igIsMouseDragging(0,-1.0)) || igIsMouseDragging(2,-1.0) {
            inf_view.view.translate((*io).MouseDelta);
        }
    }
}


fn interact(app :&mut App, draw :&Draw) {
    match &app.document.inf_view.action {
        Action::Normal(normal) => { interact_normal(app, draw, *normal); },
        Action::DrawingLine(from) => { interact_drawing(app, draw, *from); },
        Action::InsertObject(obj) => { interact_insert(app,draw,obj.clone()); },
    }
}

fn interact_normal(app :&mut App, draw :&Draw, state :NormalState) {
    unsafe {
        let io = igGetIO();
        match state {
            NormalState::SelectWindow(a) => {
                let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                if igIsMouseDragging(0,-1.0) {
                    ImDrawList_AddRect(draw.draw_list, draw.pos + a, draw.pos + b,
                                       app.config.color_u32(RailUIColorName::CanvasSelectionWindow),
                                       0.0, 0, 1.0);
                } else {
                    set_selection_window(&mut app.document, a,b);
                    app.document.inf_view.action = Action::Normal(NormalState::Default);
                }
            },
            NormalState::DragMove(typ) => {
                if igIsMouseDragging(0,-1.0) {
                    let delta = draw.view.screen_to_world_ptc((*io).MouseDelta) -
                                draw.view.screen_to_world_ptc(ImVec2 { x:0.0, y: 0.0 });
                    match typ {
                        MoveType::Continuous => { if delta.x != 0.0 || delta.y != 0.0 {
                            move_selected_objects(&mut app.document, delta); }},
                        MoveType::Grid(p) => {
                            app.document.inf_view.action = 
                                Action::Normal(NormalState::DragMove(MoveType::Grid(p + delta)));
                        },
                    }
                } else {
                    app.document.inf_view.action = Action::Normal(NormalState::Default);
                }
            }
            NormalState::Default => {
                if !(*io).KeyCtrl && igIsItemHovered(0) && igIsMouseDragging(0,-1.0) {
                    if let Some((r,_)) = app.document.get_closest(draw.pointer) {
                        if !app.document.inf_view.selection.contains(&r) {
                            app.document.inf_view.selection = std::iter::once(r).collect();
                        }
                        if app.document.inf_view.selection.iter().any(|x| matches!(x, Ref::Node(_)) || matches!(x, Ref::LineSeg(_,_))) {
                            app.document.inf_view.action = Action::Normal(NormalState::DragMove(
                                    MoveType::Grid(glm::zero())));
                        } else {
                            app.document.inf_view.action = Action::Normal(NormalState::DragMove(MoveType::Continuous));
                        }
                    } else {
                        let a = (*io).MouseClickedPos[0] - draw.pos;
                        //let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                        app.document.inf_view.action = Action::Normal(NormalState::SelectWindow(a));
                    }
                } else {
                    if igIsItemHovered(0) && igIsMouseReleased(0) {
                        if !(*io).KeyShift { app.document.inf_view.selection.clear(); }
                        if let Some((r,_)) = app.document.get_closest(draw.pointer) {
                            app.document.inf_view.selection.insert(r);
                        }
                    }
                }
            },
        }
    }

}

pub fn set_selection_window(doc :&mut Document, a :ImVec2, b :ImVec2) {
    let s = doc.get_rect(doc.inf_view.view.screen_to_world_ptc(a),
                         doc.inf_view.view.screen_to_world_ptc(b))
                .into_iter().collect();
    doc.inf_view.selection = s;
}

pub fn move_selected_objects(doc :&mut Document, delta :PtC) {
    let mut model = doc.model().clone();
    let mut changed_ptas = Vec::new();
    for id in doc.inf_view.selection.iter() {
        match id {
            Ref::Object(pta) => {
                let mut obj = model.objects.get_mut(pta).unwrap().clone();
                obj.move_to(&model, obj.loc + delta);
                let new_pta = round_coord(obj.loc);
                model.objects.remove(pta);
                model.objects.insert(new_pta,obj);
                if *pta != new_pta { changed_ptas.push((*pta,new_pta)); }
            },
            _ => {},
        }
    }

    let selection_before = doc.inf_view.selection.clone();

    for (a,b) in changed_ptas {
        doc.inf_view.selection.remove(&Ref::Object(a));
        doc.inf_view.selection.insert(Ref::Object(b));
    }

    doc.set_model(model, Some(EditClass::MoveObjects(selection_before)));
    doc.override_edit_class(EditClass::MoveObjects(doc.inf_view.selection.clone()));
}

fn interact_drawing(app :&mut App, draw :&Draw, from :Option<Pt>) {
    unsafe {
        let color = app.config.color_u32(RailUIColorName::CanvasTrackDrawing);
        // Draw preview
        if let Some(pt) = from {
            for (p1,p2) in util::route_line(pt, draw.pointer_grid) {
                ImDrawList_AddLine(draw.draw_list, draw.pos + draw.view.world_pt_to_screen(p1),
                                                   draw.pos + draw.view.world_pt_to_screen(p2),
                                              color, 2.0);
            }

            if !igIsMouseDown(0) {
                let mut new_model = app.document.model().clone();
                let mut any_lines = false;
                for (p1,p2) in util::route_line(pt,draw.pointer_grid) {
                    let unit = util::unit_step_diag_line(p1,p2);
                    for (pa,pb) in unit.iter().zip(unit.iter().skip(1)) {
                        any_lines = true;
                        new_model.linesegs.insert(util::order_ivec(*pa,*pb));
                    }
                }
                if any_lines { app.document.set_model(new_model, None); }
                app.document.inf_view.selection = std::iter::empty().collect();
                app.document.inf_view.action = Action::DrawingLine(None);
            }
        } else {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                app.document.inf_view.action = Action::DrawingLine(Some(draw.pointer_grid));
            }
        }
    }
}

fn interact_insert(app :&mut App, draw :&Draw, obj :Option<Object>) {
    unsafe {
        if let Some(mut obj) = obj {
            let moved = obj.move_to(app.document.model(),draw.pointer);
            obj.draw(draw.pos,&draw.view,draw.draw_list,
                     app.config.color_u32(RailUIColorName::CanvasSymbol),&[],&app.config);

            if let Some(err) = moved {
                let p = draw.pos + draw.view.world_ptc_to_screen(obj.loc);
                let window = ImVec2 { x: 4.0, y: 4.0 };
                ImDrawList_AddRect(draw.draw_list, p - window, p + window,
                                   app.config.color_u32(RailUIColorName::CanvasSymbolLocError),
                                   0.0,0,4.0);
            } else  {
                if igIsMouseReleased(0) {
                    app.document.edit_model(|m| {
                        m.objects.insert(round_coord(obj.loc), obj.clone());
                        None
                    });
                }
            }
        }
    }
}

fn inf_toolbar(app :&mut App) {
    unsafe  {
    if toolbar_button(const_cstr!("select (A)").as_ptr(), 
                      'A' as _,  matches!(app.document.inf_view.action, Action::Normal(_))) {
        app.document.inf_view.action = Action::Normal(NormalState::Default);
    }
    igSameLine(0.0,-1.0);
    if toolbar_button(const_cstr!("insert (S)").as_ptr(), 
                      'S' as _,  matches!(app.document.inf_view.action, Action::InsertObject(_))) {
        app.document.inf_view.action = Action::InsertObject(None);
    }
    igSameLine(0.0,-1.0);
    if toolbar_button(const_cstr!("draw (D)").as_ptr(), 
                      'A' as _,  matches!(app.document.inf_view.action, Action::DrawingLine(_))) {
        app.document.inf_view.action = Action::DrawingLine(None);
    }
    }
}

fn toolbar_button(name :*const i8, char :i8, selected :bool) -> bool {
        unsafe {
        if selected {
            let c1 = ImVec4 { x: 0.4, y: 0.65,  z: 0.4, w: 1.0 };
            let c2 = ImVec4 { x: 0.5, y: 0.85, z: 0.5, w: 1.0 };
            let c3 = ImVec4 { x: 0.6, y: 0.9,  z: 0.6, w: 1.0 };
            igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        }
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        if selected {
            igPopStyleColor(3);
        }
        clicked
    }
}

fn context_menu(doc :&mut Document, draw :&Draw, preview_route :&mut Option<usize>) {
    unsafe {
    if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
        context_menu_contents(doc, preview_route);
        igEndPopup();
    }

    if igIsItemHovered(0) && igIsMouseClicked(1, false) {
        if let Some((r,_)) = doc.get_closest(draw.pointer) {
            if !doc.inf_view.selection.contains(&r) {
                doc.inf_view.selection = std::iter::once(r).collect();
            }
        }
        igOpenPopup(const_cstr!("ctx").as_ptr());
    }
    }
}


fn context_menu_contents(doc :&mut Document, preview_route :&mut Option<usize>) {
    unsafe {

        widgets::show_text(&format!("selection: {:?}", doc.inf_view.selection));
        //
        // TODO cache some info about selection? In case it is very big and we need to know
        // every frame whether it contains a Node or not.
        // 

        if doc.inf_view.selection.len() == 1 {
            if let Some(Ref::Node(pt)) = doc.inf_view.selection.iter().cloned().nth(0) {
                menus::node_editor(doc, pt);
            }
        }

        widgets::sep();
        if !doc.inf_view.selection.is_empty() {
            if igSelectable(const_cstr!("Delete").as_ptr(), false, 0 as _, ImVec2::zero()) {
                delete_selection(doc);
            }
        }

        widgets::sep();
        let mut dispatch_action = None;
        if doc.inf_view.selection.len() == 1 {


            // Object menu
            if let Some(Ref::Object(pta)) = doc.inf_view.selection.iter().cloned().nth(0) {
                menus::object_menu(doc, pta);
            }

            if let Some(il) = doc.data().interlocking.as_ref() {
                if let Some(Ref::Node(pt)) = doc.inf_view.selection.iter().cloned().nth(0) {
                    if let Some(rs) = il.boundary_routes.get(&pt) {
                        let (preview,action) = menus::route_selector(il,rs);
                        *preview_route = preview;
                        dispatch_action = action;
                    }
                }
                if let Some(Ref::Object(pta)) = doc.inf_view.selection.iter().cloned().nth(0) {
                    if let Some(rs) = il.signal_routes.get(&pta) {
                        let (preview,action) = menus::route_selector(il,rs);
                        *preview_route = preview;
                        dispatch_action = action;
                    }
                }
            }
        }
        // This can be moved inside the route_selector?
        if let Some(route_id) = dispatch_action {
            start_route(doc, route_id);
        }

    }
}


fn delete_selection(doc :&mut Document) {
    let mut new_model = doc.model().clone();
    for x in doc.inf_view.selection.drain() {
        new_model.delete(x);
    }
    doc.set_model(new_model, None);
}

fn start_route(doc :&mut Document, route_id :usize) {
    //   if let Some(il) = doc.data().interlocking.as_ref() {
    //   let mut model = doc.model().clone();

    //   if doc.dispatch_view.is_some() {
    //   }

    //   let (dispatch_idx,time,play) = app.doc.unwrap_or_else(|| {
    //       model.dispatches.push_back(Default::default()); // empty dispatch
    //       let d = (model.dispatches.len()-1, 0.0, true);
    //       self.active_dispatch = Some(d);
    //       d
    //   });

    //   let dispatch = model.dispatches.get_mut(dispatch_idx).unwrap();
    //   let cmd = match (il.routes[route_idx].0).entry {
    //       rolling_inf::RouteEntryExit::Boundary(_) =>
    //           Command::Train { route: route_idx, vehicle: 0 },
    //       rolling_inf::RouteEntryExit::Signal(_) | rolling_inf::RouteEntryExit::SignalTrigger {..
    //           Command::Route { route: route_idx },
    //   };
    //   dispatch.insert(time as f64, cmd);
    //   doc.set_model(model,None);
    //   //println!("DISPATCHES: {:?}", doc.get_undoable().get().dispatches);
}
