use ordered_float::OrderedFloat;
use backend_glfw::imgui::*;
use const_cstr::*;
use std::ffi::CString;

use crate::document::*;
use crate::document::analysis::*;
use crate::app::*;
use crate::config::*;
use crate::gui::widgets;
use crate::gui::plan;
use crate::gui::diagram::diagram_view;
use crate::util::VecMap;
use crate::gui::diagram::*;

pub fn dispatch_view(config :&Config, analysis :&mut Analysis, dv :&mut DispatchView) -> Option<Option<DispatchView>> {
    let mut new_dispatch :Option<Option<DispatchView>> = None;
    let sel = dispatch_select_bar(&Some(*dv), analysis);
    new_dispatch = sel.or(new_dispatch);

    match dv {
        DispatchView::Manual(manual) => {
            let graph = analysis.data().dispatch.vecmap_get(manual.dispatch_idx);
            if let Some((_gen,graph)) = graph {
                if let Some(action) = diagram_view(config, analysis, manual, graph) {
                    analysis.edit_model(|m| {
                        match action {
                            DiagramViewAction::DeleteCommand { id } => {
                                m.dispatches.get_mut(manual.dispatch_idx)?.commands.retain(|(x,_)| *x != id);
                            },
                            DiagramViewAction::MoveCommand { idx, id, t } => {
                                let commands = &mut m.dispatches.get_mut(manual.dispatch_idx)?.commands;
                                for (c_id,(c_t,_)) in commands.iter_mut() {
                                    if *c_id == id { *c_t = t; }
                                }
                                commands.sort_by_key(|(_,(t,_))| OrderedFloat(*t));
                            }
                        };
                        None
                    });
                }
            }
        },
        DispatchView::Auto(auto) => {
            let new_auto = plan::edit_plan(config, analysis, auto);
            new_dispatch = new_auto.or(new_dispatch);

            if let Some(manual) = &mut auto.dispatch {
                let graph = analysis.data().plandispatches.get(&auto.plan_idx)
                    .and_then(|p| p.vecmap_get(manual.dispatch_idx));
                if let Some((_gen,graph)) = graph {
                    diagram_view(config, analysis, manual, graph);
                }
            }
        },
    }

    new_dispatch
}

/// Select a new dispatch view from manual or auto dispatches already existing in model
pub fn dispatch_select_bar(dispatch_view :&Option<DispatchView>, analysis :&mut Analysis) -> Option<Option<DispatchView>> {
    unsafe {
        let mut new_dispatch_auto = None;
        let mut retval = None;

        igPushItemWidth(250.0);
        let current_name = match dispatch_view {
            None => CString::new("None").unwrap(),
            Some(DispatchView::Manual(ManualDispatchView { dispatch_idx, .. })) => 
                CString::new(format!("Dispatch {}",dispatch_idx)).unwrap(),
            Some(DispatchView::Auto(AutoDispatchView { plan_idx , .. })) => 
                CString::new(format!("Plan {}",plan_idx)).unwrap(),
        };

        let curr_manual = if let Some(DispatchView::Manual(ManualDispatchView { dispatch_idx , ..})) = &dispatch_view {
            Some(dispatch_idx) } else { None };
        let curr_auto = if let Some(DispatchView::Auto(AutoDispatchView { plan_idx , ..})) = &dispatch_view {
            Some(plan_idx) } else { None };

        let comboflag = ImGuiComboFlags__ImGuiComboFlags_HeightLarge;
        if igBeginCombo(const_cstr!("##sel").as_ptr(), current_name.as_ptr(), comboflag as _) {

            if igSelectable(const_cstr!("None").as_ptr(), dispatch_view.is_none(), 0 as _, ImVec2::zero()) {
                retval = Some(None);
            }

            widgets::sep();

            igPushIDInt(1);
            let mut any = false;
            for (id,_) in analysis.model().dispatches.iter() {
                any = true;
                igPushIDInt(*id as _);

                if igSelectable(const_cstr!("##smanu").as_ptr(), Some(id) == curr_manual, 0 as _, ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Manual(ManualDispatchView::new(*id)))); 
                }

                igSameLine(0.0,-1.0); widgets::show_text(&format!("Dispatch {}", id));

                igPopID();
            }
            if !any { widgets::show_text("No dispatches."); }
            igPopID();

            igSpacing();
            if igButton(const_cstr!("\u{f0fe} Manual").as_ptr(), ImVec2::zero()) {
                new_dispatch_auto = Some(false);
            }
            igSpacing();

            widgets::sep();
            igPushIDInt(2);
            let mut any = false;
            for (id,_) in analysis.model().plans.iter() {
                any = true;
                igPushIDInt(*id as _);

                if igSelectable(const_cstr!("##sauto").as_ptr(), Some(id) == curr_auto, 0 as _, ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Auto(AutoDispatchView {
                        plan_idx: *id,
                        dispatch: None,
                        action: PlanViewAction::None,
                    })));
                }
                igSameLine(0.0,-1.0); widgets::show_text(&format!("Plan {}", id));

                igPopID();
            }
            if !any { widgets::show_text("No plans."); }
            igPopID();


            igSpacing();
            if igButton(const_cstr!("\u{f0fe} Auto").as_ptr(), ImVec2::zero()) {
                new_dispatch_auto = Some(true);
            }
            igSpacing();

            igEndCombo();
        }
        igPopItemWidth();

        if new_dispatch_auto == Some(false) {
            // Create new dispatch and set it to current
            let mut model = analysis.model().clone();
            let id = model.dispatches.insert(Default::default());
            analysis.set_model(model, None);

            retval = Some(Some(DispatchView::Manual(ManualDispatchView::new(id))));
        }

        if new_dispatch_auto == Some(true) {
            // Create new plan and set it to current
            let mut model = analysis.model().clone();
            let id = model.plans.insert(Default::default());
            analysis.set_model(model, None);

            retval = Some(Some(DispatchView::Auto(AutoDispatchView {
                plan_idx: id,
                dispatch: None,
                action: PlanViewAction::None,
            })));
        }

        retval
    }
}
