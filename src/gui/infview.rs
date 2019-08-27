use const_cstr::*;
use matches::*;
use backend_glfw::imgui::*;

use crate::app::App;
use crate::document::*;
use crate::document::infview::*;
use crate::document::view::*;
use crate::document::interlocking::*;
use crate::document::model::*;
use crate::gui::widgets;
use crate::gui::widgets::Draw;
use crate::gui::menus;
use crate::gui::draw_inf;
use crate::config::RailUIColorName;


pub fn inf_view(app :&mut App) {
    inf_toolbar(app);

    // TODO improve API here
    //widgets::Canvas::new(const_cstr!("inf").as_ptr(), View::default())
    //    .background(app.config.color_u32(RailUIColorName::CanvasBackground))
    //    .draw(|d| {

    //});

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

fn scroll(inf_view :&mut InfView) { }


fn interact(app :&mut App, draw :&Draw) {
    match &mut app.document.inf_view.action {
        _ => {},
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


fn delete_selection(doc :&mut Document) {}
fn start_route(doc :&mut Document, route_id :usize) {}
