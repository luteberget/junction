use const_cstr::*;
use backend_glfw::imgui::*;

use crate::app::*;
use crate::document::model::*;
use crate::document::*;
use crate::gui::widgets;

pub fn plan_view(app :&mut App) {
    if let Some(DispatchView::Auto(AutoDispatchView { plan_idx, .. })) = &app.document.dispatch_view {
        edit_plan(app, *plan_idx);
    }
}

pub fn edit_plan(app :&mut App, plan_idx :ListId) {
    unsafe {
        igSameLine(0.0,-1.0);


        if igButton(const_cstr!("(+) Train").as_ptr(), ImVec2::zero()) {
            let default_train = app.document.model().vehicles.iter().next().map(|(id,_)| *id);
            app.document.edit_model(|m| {
                m.plans.get_mut(plan_idx).unwrap().trains.insert((default_train, ImShortGenList::new()));
                None
            });
        }

        widgets::sep();

        igDummy(ImVec2 { x: 0.0, y: 7.0 } );

        let toposort = get_toposort(app, plan_idx);

        if let Some(plan) = app.document.model().plans.get(plan_idx) {
            for (train_id,(vehicle_ref,visits)) in plan.trains.iter() {
                igPushIDInt(*train_id as _);

                igPushItemWidth(125.0);
                select_train_combo(app.document.model());
                igPopItemWidth();

                // If there are no visits, draw a yellow suggested visit box 
                if visits.iter().next().is_none() {
                    //igSameLine(125.0 + igGetFrameHeight(),-1.0);
                    igSameLine(0.0,-1.0);

                    yellow_button(const_cstr!("?").as_ptr());
                }

                igDummy(ImVec2 { x: 0.0, y: 7.0 } );
                widgets::sep();
                igDummy(ImVec2 { x: 0.0, y: 7.0 } );

                igPopID();
            }
        }
    }
}

pub fn select_train_combo(model :&Model) -> Option<ListId> {
    let mut v = None;
    unsafe {
        if igBeginCombo(const_cstr!("##chtr").as_ptr(), const_cstr!("Train").as_ptr(), 0) {
            v = v.or(select_train(model));
            igEndCombo();
        }
    }
    v
}

pub fn select_train(model :&Model) -> Option<ListId> {
    let mut retval = None;
    let mut any = false;
    unsafe {
        for (i,v) in model.vehicles.iter() {
            any = true;
            igPushIDInt(*i as _);
            if igSelectable(const_cstr!("##vh").as_ptr(), false, 0 as _, ImVec2::zero()) {
                retval = Some(*i);
            }
            igSameLine(0.0,-1.0);
            widgets::show_text(&v.name);
            igPopID();
        }
    }

    if !any { widgets::show_text("No vehicles."); }
    retval
}

fn get_toposort(app :&mut App, plan_idx :usize) -> Vec<Vec<(ListId,ListId)>> {
    Vec::new()
}

fn yellow_button(name :*const i8) -> bool{
    unsafe {
        let c1 = ImVec4 { x: 1.0, y: 0.95,  z: 0.2, w: 0.4 };
        let c2 = ImVec4 { x: 1.0, y: 1.0, z: 0.22, w: 0.4 };
        let c3 = ImVec4 { x: 1.0, y: 1.0,  z: 0.24, w: 0.4 };
        igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        if igIsItemHovered(0) {
            igBeginTooltip();
            widgets::show_text("Train has no visits. Right click on the infrastructure to add visits to this train.");
            igEndTooltip();
        }
        igPopStyleColor(3);
        clicked
    }
}



