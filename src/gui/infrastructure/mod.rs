pub mod draw;
pub mod menus;

use const_cstr::*;
use matches::*;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;

use crate::util;
use crate::app::App;
use crate::config::*;
use crate::document::*;
use crate::document::infview::*;
use crate::document::view::*;
use crate::document::interlocking::*;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::document::objects::*;
use crate::gui::widgets;
use crate::gui::widgets::Draw;
use crate::config::RailUIColorName;


pub fn inf_view(config :&Config, 
                analysis :&mut Analysis,
                inf_view :&mut InfView,
                dispatch_view :&mut Option<DispatchView>) {
    unsafe {
    let pos_before : ImVec2 = igGetCursorPos_nonUDT2().into();

    let size = igGetContentRegionAvail_nonUDT2().into();
    widgets::canvas(size,
                    config.color_u32(RailUIColorName::CanvasBackground),
                    const_cstr!("railwaycanvas").as_ptr(),
                    inf_view.view.clone(),
                    |draw| {
        scroll(inf_view);
        let mut preview_route = None;
        context_menu(analysis, inf_view, dispatch_view, draw, &mut preview_route);
        interact(config, analysis, inf_view, draw);
        draw_inf(config, analysis, inf_view, draw, preview_route);
        Some(())
    });

    let pos_after = igGetCursorPos_nonUDT2().into();
    let framespace = igGetFrameHeightWithSpacing() - igGetFrameHeight();
    igSetCursorPos(pos_before + ImVec2 { x: 2.0*framespace, y: 2.0*framespace });
    inf_toolbar(inf_view);
    igSetCursorPos(pos_after);
    }
}

fn draw_inf(config :&Config, analysis :&Analysis, inf_view :&InfView, 
            draw :&Draw, preview_route :Option<usize>) {

    draw::base(config, analysis, inf_view, draw);
    if let Some(r) = preview_route { draw::route(config, analysis, draw, r); }

    //draw::state(app,draw);
    //draw::trains(app,draw);
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


fn interact(config :&Config, analysis :&mut Analysis, inf_view :&mut InfView, draw :&Draw) {
    match &inf_view.action {
        Action::Normal(normal) => { interact_normal(config, analysis, inf_view, draw, *normal); },
        Action::DrawingLine(from) => { interact_drawing(config, analysis, inf_view, draw, *from); },
        Action::InsertObject(obj) => { interact_insert(config,analysis,draw,obj.clone()); },
    }
}

fn interact_normal(config :&Config, analysis :&mut Analysis, 
                   inf_view :&mut InfView, draw :&Draw, state :NormalState) {
    // config
    // inf_view
    // analysis
    unsafe {
        let io = igGetIO();
        match state {
            NormalState::SelectWindow(a) => {
                let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                if igIsMouseDragging(0,-1.0) {
                    ImDrawList_AddRect(draw.draw_list, draw.pos + a, draw.pos + b,
                                       config.color_u32(RailUIColorName::CanvasSelectionWindow),
                                       0.0, 0, 1.0);
                } else {
                    set_selection_window(inf_view, analysis, a,b);
                    inf_view.action = Action::Normal(NormalState::Default);
                }
            },
            NormalState::DragMove(typ) => {
                if igIsMouseDragging(0,-1.0) {
                    let delta = draw.view.screen_to_world_ptc((*io).MouseDelta) -
                                draw.view.screen_to_world_ptc(ImVec2 { x:0.0, y: 0.0 });
                    match typ {
                        MoveType::Continuous => { if delta.x != 0.0 || delta.y != 0.0 {
                            move_selected_objects(analysis, inf_view, delta); }},
                        MoveType::Grid(p) => {
                            inf_view.action = 
                                Action::Normal(NormalState::DragMove(MoveType::Grid(p + delta)));
                        },
                    }
                } else {
                    inf_view.action = Action::Normal(NormalState::Default);
                }
            }
            NormalState::Default => {
                if !(*io).KeyCtrl && igIsItemHovered(0) && igIsMouseDragging(0,-1.0) {
                    if let Some((r,_)) = analysis.get_closest(draw.pointer) {
                        if !inf_view.selection.contains(&r) {
                            inf_view.selection = std::iter::once(r).collect();
                        }
                        if inf_view.selection.iter().any(|x| matches!(x, Ref::Node(_)) || matches!(x, Ref::LineSeg(_,_))) {
                            inf_view.action = Action::Normal(NormalState::DragMove(
                                    MoveType::Grid(glm::zero())));
                        } else {
                            inf_view.action = Action::Normal(NormalState::DragMove(MoveType::Continuous));
                        }
                    } else {
                        let a = (*io).MouseClickedPos[0] - draw.pos;
                        //let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                        inf_view.action = Action::Normal(NormalState::SelectWindow(a));
                    }
                } else {
                    if igIsItemHovered(0) && igIsMouseReleased(0) {
                        if !(*io).KeyShift { inf_view.selection.clear(); }
                        if let Some((r,_)) = analysis.get_closest(draw.pointer) {
                            inf_view.selection.insert(r);
                        }
                    }
                }
            },
        }
    }

}

pub fn set_selection_window(inf_view :&mut InfView, analysis :&Analysis, a :ImVec2, b :ImVec2) {
    let s = analysis.get_rect(inf_view.view.screen_to_world_ptc(a),
                         inf_view.view.screen_to_world_ptc(b))
                .into_iter().collect();
    inf_view.selection = s;
}

pub fn move_selected_objects(analysis :&mut Analysis, inf_view :&mut InfView, delta :PtC) {
    let mut model = analysis.model().clone();
    let mut changed_ptas = Vec::new();
    for id in inf_view.selection.iter() {
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

    let selection_before = inf_view.selection.clone();

    for (a,b) in changed_ptas {
        inf_view.selection.remove(&Ref::Object(a));
        inf_view.selection.insert(Ref::Object(b));
    }

    analysis.set_model(model, Some(EditClass::MoveObjects(selection_before)));
    analysis.override_edit_class(EditClass::MoveObjects(inf_view.selection.clone()));
}

fn interact_drawing(config :&Config, analysis :&mut Analysis, inf_view :&mut InfView, 
                    draw :&Draw, from :Option<Pt>) {
    unsafe {
        let color = config.color_u32(RailUIColorName::CanvasTrackDrawing);
        // Draw preview
        if let Some(pt) = from {
            for (p1,p2) in util::route_line(pt, draw.pointer_grid) {
                ImDrawList_AddLine(draw.draw_list, draw.pos + draw.view.world_pt_to_screen(p1),
                                                   draw.pos + draw.view.world_pt_to_screen(p2),
                                              color, 2.0);
            }

            if !igIsMouseDown(0) {
                let mut new_model = analysis.model().clone();
                let mut any_lines = false;
                for (p1,p2) in util::route_line(pt,draw.pointer_grid) {
                    let unit = util::unit_step_diag_line(p1,p2);
                    for (pa,pb) in unit.iter().zip(unit.iter().skip(1)) {
                        any_lines = true;
                        new_model.linesegs.insert(util::order_ivec(*pa,*pb));
                    }
                }
                if any_lines { analysis.set_model(new_model, None); }
                inf_view.selection = std::iter::empty().collect();
                inf_view.action = Action::DrawingLine(None);
            }
        } else {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                inf_view.action = Action::DrawingLine(Some(draw.pointer_grid));
            }
        }
    }
}

fn interact_insert(config :&Config, analysis :&mut Analysis, 
                   draw :&Draw, obj :Option<Object>) {
    unsafe {
        if let Some(mut obj) = obj {
            let moved = obj.move_to(analysis.model(),draw.pointer);
            obj.draw(draw.pos,&draw.view,draw.draw_list,
                     config.color_u32(RailUIColorName::CanvasSymbol),&[],&config);

            if let Some(err) = moved {
                let p = draw.pos + draw.view.world_ptc_to_screen(obj.loc);
                let window = ImVec2 { x: 4.0, y: 4.0 };
                ImDrawList_AddRect(draw.draw_list, p - window, p + window,
                                   config.color_u32(RailUIColorName::CanvasSymbolLocError),
                                   0.0,0,4.0);
            } else  {
                if igIsMouseReleased(0) {
                    analysis.edit_model(|m| {
                        m.objects.insert(round_coord(obj.loc), obj.clone());
                        None
                    });
                }
            }
        }
    }
}

fn inf_toolbar(inf_view :&mut InfView) {
    unsafe  {
    if toolbar_button(const_cstr!("select (A)").as_ptr(), 
                      'A' as _,  matches!(inf_view.action, Action::Normal(_))) {
        inf_view.action = Action::Normal(NormalState::Default);
    }
    igSameLine(0.0,-1.0);
    if toolbar_button(const_cstr!("insert (S)").as_ptr(), 
                      'S' as _,  matches!(inf_view.action, Action::InsertObject(_))) {
        inf_view.action = Action::InsertObject(None);
    }
    igSameLine(0.0,-1.0);
    if toolbar_button(const_cstr!("draw (D)").as_ptr(), 
                      'A' as _,  matches!(inf_view.action, Action::DrawingLine(_))) {
        inf_view.action = Action::DrawingLine(None);
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

fn context_menu(analysis :&mut Analysis, 
                inf_view :&mut InfView,
                dispatch_view :&mut Option<DispatchView>,
                draw :&Draw, preview_route :&mut Option<usize>) {
    unsafe {
    if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
        context_menu_contents(analysis, inf_view, dispatch_view, preview_route);
        igEndPopup();
    }

    if igIsItemHovered(0) && igIsMouseClicked(1, false) {
        if let Some((r,_)) = analysis.get_closest(draw.pointer) {
            if !inf_view.selection.contains(&r) {
                inf_view.selection = std::iter::once(r).collect();
            }
        }
        igOpenPopup(const_cstr!("ctx").as_ptr());
    }
    }
}

fn context_menu_contents(analysis :&mut Analysis, inf_view :&mut InfView,
                         dispatch_view :&mut Option<DispatchView>,
                         preview_route :&mut Option<usize>) {
    unsafe {
    widgets::show_text(&format!("selection: {:?}", inf_view.selection));
    widgets::sep();
    if !inf_view.selection.is_empty() {
        if igSelectable(const_cstr!("Delete").as_ptr(), false, 0 as _, ImVec2::zero()) {
            delete_selection(analysis, inf_view);
        }
    }
    widgets::sep();
    if inf_view.selection.len() == 1 {
        let thing = inf_view.selection.iter().nth(0).cloned().unwrap();
        context_menu_single(analysis, dispatch_view ,thing,preview_route);
    }
    }
}

fn context_menu_single(analysis :&mut Analysis, 
                       dispatch_view :&mut Option<DispatchView>,
                       thing :Ref, preview_route :&mut Option<usize>) {

    // Node editor
    if let Ref::Node(pt) = thing { 
        menus::node_editor(analysis, pt);
        widgets::sep();
    }

    // Object editor
    if let Ref::Object(pta) = thing { 
        menus::object_menu(analysis, pta);
        widgets::sep();
    }

    // Manual dispatch from boundaries and signals
    let action = menus::route_selector(analysis, dispatch_view, thing, preview_route);
    if let Some(routespec) = action {
        start_route(analysis, dispatch_view, routespec);
    }
    widgets::sep();

    // Add visits to auto dispatch
    menus::add_plan_visit(analysis, dispatch_view, thing);
}


fn delete_selection(analysis :&mut Analysis, inf_view :&mut InfView) {
    let mut new_model = analysis.model().clone();
    for x in inf_view.selection.drain() {
        new_model.delete(x);
    }
    analysis.set_model(new_model, None);
}

fn start_route(analysis :&mut Analysis, dispatch_view :&mut Option<DispatchView>, cmd :Command) {
    let mut model = analysis.model().clone();

    let (dispatch_idx,time) = match &dispatch_view {
        Some(DispatchView::Manual(m)) => (m.dispatch_idx, m.time),
        None | Some(DispatchView::Auto(_)) => {
            let dispatch_idx = model.dispatches.insert(Default::default());
            let time = 0.0;
            *dispatch_view = Some(DispatchView::Manual(ManualDispatchView {
                dispatch_idx: dispatch_idx, time: 0.0, play: true
            }));
            (dispatch_idx,time)
        },
    };

    let dispatch = model.dispatches.get_mut(dispatch_idx).unwrap();
    dispatch.insert(time as f64, cmd);
    analysis.set_model(model, None);
}
