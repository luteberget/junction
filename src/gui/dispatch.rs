use backend_glfw::imgui::*;
use const_cstr::*;
use std::ffi::CString;

use crate::document::*;
use crate::app::*;
use crate::gui::widgets;

pub fn dispatch_view(app :&mut App) {
    dispatch_select_bar(app);
    match &app.document.dispatch_view {
        Some(DispatchView::Manual(m)) => {
            //diagram_view(app);
        },
        Some(DispatchView::Auto(a)) => {
            //plan_view(app);
            if let Some(m) = &a.dispatch {
                //diagram_view();
            }
        },
        None => {}, // should not happen
    }
}

pub fn dispatch_select_bar(app :&mut App) {
    unsafe {
        if let Some(new) = dispatch_select(app) {
            app.document.dispatch_view = new;
        }

        igSameLine(0.0, -1.0);
        if igButton(const_cstr!("(+) Manual").as_ptr(), ImVec2::zero()) {
            // TODO
        }

        igSameLine(0.0, -1.0);
        if igButton(const_cstr!("(+) Auto").as_ptr(), ImVec2::zero()) {
            // TODO 
        }
    }
}

/// Select a new dispatch view from manual or auto dispatches already existing in model
pub fn dispatch_select(app :&mut App) -> Option<Option<DispatchView>> {
    unsafe {
        let mut retval = None;
        igPushItemWidth(250.0);
        let current_name = match app.document.dispatch_view {
            None => CString::new("None").unwrap(),
            Some(DispatchView::Manual(ManualDispatchView { dispatch_idx, .. })) => 
                CString::new(format!("Dispatch {}",dispatch_idx)).unwrap(),
            Some(DispatchView::Auto(AutoDispatchView { plan_idx , .. })) => 
                CString::new(format!("Plan {}",plan_idx)).unwrap(),
        };

        let mut curr_manual = if let Some(DispatchView::Manual(ManualDispatchView { dispatch_idx , ..})) = &app.document.dispatch_view {
            Some(dispatch_idx) } else { None };
        let mut curr_auto = if let Some(DispatchView::Auto(AutoDispatchView { plan_idx , ..})) = &app.document.dispatch_view {
            Some(plan_idx) } else { None };

        if igBeginCombo(const_cstr!("##sel").as_ptr(), current_name.as_ptr(), 0) {

            if igSelectable(const_cstr!("None").as_ptr(), app.document.dispatch_view.is_none(), 0 as _, ImVec2::zero()) {
                retval = Some(None);
            }

            widgets::sep();

            igPushIDInt(1);
            let mut any = false;
            for (id,_) in app.document.model().dispatches.iter() {
                any = true;
                igPushIDInt(*id as _);

                if igSelectable(const_cstr!("##smanu").as_ptr(), Some(id) == curr_manual, 0 as _, ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Manual(ManualDispatchView {
                        dispatch_idx: *id,
                        time: 0.0,
                        play: false,
                    })));
                }

                igSameLine(0.0,-1.0); widgets::show_text(&format!("Dispatch {}", id));

                igPopID();
            }
            if !any { widgets::show_text("No dispatches."); }
            igPopID();

            widgets::sep();
            igPushIDInt(2);
            let mut any = false;
            for (id,_) in app.document.model().plans.iter() {
                any = true;
                igPushIDInt(*id as _);

                if igSelectable(const_cstr!("##sauto").as_ptr(), Some(id) == curr_auto, 0 as _, ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Auto(AutoDispatchView {
                        plan_idx: *id,
                        dispatch: None,
                    })));
                }
                igSameLine(0.0,-1.0); widgets::show_text(&format!("Plan {}", id));

                igPopID();
            }
            if !any { widgets::show_text("No plans."); }
            igPopID();



            igEndCombo();
        }
        igPopItemWidth();
        retval
    }
}
