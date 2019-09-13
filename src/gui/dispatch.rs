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
use crate::gui::widgets::Draw;
use crate::document::infview::InfView;

pub fn dispatch_view(config :&Config, inf_canvas :Option<&Draw>, inf_view :&InfView,
                     analysis :&mut Analysis, dv :&mut DispatchView) -> Option<Option<DispatchView>> {
    let mut new_dispatch :Option<Option<DispatchView>> = None;
    let sel = dispatch_select_bar(&Some(*dv), analysis);
    new_dispatch = sel.or(new_dispatch);

    match dv {
        DispatchView::Manual(manual) => {
            let graph = analysis.data().dispatch.vecmap_get(manual.dispatch_idx);
            if let Some((_gen,graph)) = graph {
                unsafe { igSameLine(0.0, -1.0); }
                if let Some(action) = diagram_view(config, inf_canvas, inf_view, analysis, manual, graph) {
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

            if !(manual.dispatch_idx < analysis.model().dispatches.data().len()) {
                new_dispatch = Some(None);
            }
        },
        DispatchView::Auto(auto) => {
            let new_auto = plan::edit_plan(config, inf_canvas, inf_view, analysis, auto);
            new_dispatch = new_auto.or(new_dispatch);

            if let Some(manual) = &mut auto.dispatch {
                let graph = analysis.data().plandispatches.get(&auto.plan_idx)
                    .and_then(|p| p.vecmap_get(manual.dispatch_idx));
                if let Some((_gen,graph)) = graph {
                    diagram_view(config, inf_canvas, inf_view, analysis, manual, graph);
                } else {
                    // Plan doesn't exist anymore.
                    auto.dispatch = None;
                }
            }
        },
    }

    new_dispatch
}

pub enum Action {
    DispatchName(usize,String),
    PlanName(usize, String),
    DeleteDispatch(usize),
    DeletePlan(usize),
}

/// Select a new dispatch view from manual or auto dispatches already existing in model
pub fn dispatch_select_bar(dispatch_view :&Option<DispatchView>, analysis :&mut Analysis) -> Option<Option<DispatchView>> {
    unsafe {
        let mut new_dispatch_auto = None;
        let mut retval = None;
        let mut action = None;

        let current_name = match dispatch_view {
            None => CString::new("\u{f2f2} None").unwrap(),
            Some(DispatchView::Manual(ManualDispatchView { dispatch_idx, .. })) => {
                if let Some(d) = analysis.model().dispatches.get(*dispatch_idx) {
                    CString::new(format!("\u{f4fd} {}",&d.name)).unwrap()
                } else {
                    CString::new(format!("\u{f4fd} Dispatch ?")).unwrap()
                }
            }
            Some(DispatchView::Auto(AutoDispatchView { plan_idx , .. })) =>  {
                if let Some(p) = analysis.model().plans.get(*plan_idx) {
                    CString::new(format!("\u{f0d0} {}",&p.name)).unwrap()
                } else {
                    CString::new(format!("\u{f0d0} Plan ?")).unwrap()
                }
            }
        };

        igPushItemWidth(250.0);
        let curr_manual = if let Some(DispatchView::Manual(
                ManualDispatchView { dispatch_idx , ..})) = &dispatch_view {
            Some(dispatch_idx) } else { None };
        let curr_auto = if let Some(DispatchView::Auto(
                AutoDispatchView { plan_idx , ..})) = &dispatch_view {
            Some(plan_idx) } else { None };

        let comboflag = ImGuiComboFlags__ImGuiComboFlags_HeightLarge;
        if igBeginCombo(const_cstr!("##sel").as_ptr(), current_name.as_ptr(), comboflag as _) {

            if igSelectable(const_cstr!("None").as_ptr(), 
                            dispatch_view.is_none(), 0 as _, ImVec2::zero()) {
                retval = Some(None);
            }

            widgets::sep();

            igPushIDInt(1);
            let mut any = false;
            for (id,d) in analysis.model().dispatches.iter() {
                any = true;
                igPushIDInt(*id as _);

                igAlignTextToFramePadding();
                widgets::show_text("\u{f4fd}");

                igSameLine(0.0,-1.0); 
                if let Some(new_name) = widgets::edit_text(const_cstr!("##dnm").as_ptr(), 
                                                           d.name.clone()) {
                    action = Some(Action::DispatchName(*id, new_name));
                }
                igSameLine(0.0,-1.0); 
                if igButton(const_cstr!("\u{f2ed}").as_ptr(), ImVec2::zero()) {
                    action = Some(Action::DeleteDispatch(*id));
                }
                igSameLine(0.0,-1.0); 
                if igButton(const_cstr!("\u{f07c}").as_ptr(), ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Manual(ManualDispatchView::new(*id)))); 
                }

                igPopID();
            }
            if !any { widgets::show_text("No dispatches."); }
            igPopID();

            igSpacing();
            if igButton(const_cstr!("\u{f0fe}\u{f4fd} Manual").as_ptr(), ImVec2::zero()) {
                new_dispatch_auto = Some(false);
            }
            igSpacing();

            widgets::sep();
            igPushIDInt(2);
            let mut any = false;
            for (id,p) in analysis.model().plans.iter() {
                any = true;
                igPushIDInt(*id as _);

                igAlignTextToFramePadding();
                widgets::show_text("\u{f0d0}");

                igSameLine(0.0,-1.0); 
                if let Some(new_name) = widgets::edit_text(const_cstr!("##pnm").as_ptr(), 
                                                           p.name.clone()) {
                    action = Some(Action::PlanName(*id, new_name));
                }

                igSameLine(0.0,-1.0);
                if igButton(const_cstr!("\u{f2ed}").as_ptr(), ImVec2::zero()) {
                    action = Some(Action::DeletePlan(*id));
                }
                igSameLine(0.0,-1.0); 
                if igButton(const_cstr!("\u{f07c}").as_ptr(), ImVec2::zero()) {
                    retval = Some(Some(DispatchView::Auto(AutoDispatchView {
                        plan_idx: *id,
                        dispatch: None,
                        action: PlanViewAction::None,
                    })));
                }

                igPopID();
            }
            if !any { widgets::show_text("No plans."); }
            igPopID();


            igSpacing();
            if igButton(const_cstr!("\u{f0fe}\u{f0d0} Auto").as_ptr(), ImVec2::zero()) {
                new_dispatch_auto = Some(true);
            }
            igSpacing();

            igEndCombo();
        }

        igPopItemWidth();

        if igIsItemHovered(0) {
            igBeginTooltip();
            widgets::show_text("Add automatic or manual dispatching.");
            igEndTooltip();
        }

        match action {
            Some(Action::DispatchName(id,name)) => {
                analysis.edit_model(|m| {
                    if let Some(d) = m.dispatches.get_mut(id) { d.name = name; }
                    Some(model::EditClass::DispatchName(id))
                });
            },
            Some(Action::PlanName(id,name)) => {
                analysis.edit_model(|m| {
                    if let Some(p) = m.plans.get_mut(id) { p.name = name; }
                    Some(model::EditClass::PlanName(id))
                });
            },
            Some(Action::DeleteDispatch(id)) => {
                analysis.edit_model(|m| {
                    m.dispatches.remove(id);
                    None
                });
            }
            Some(Action::DeletePlan(id)) => {
                analysis.edit_model(|m| {
                    m.plans.remove(id);
                    None
                });
            }
            _ => {},
        }

        if new_dispatch_auto == Some(false) {
            // Create new dispatch and set it to current
            let mut model = analysis.model().clone();
            let dispatch_name = format!("Dispatch {}", model.dispatches.next_id()+1);
            let id = model.dispatches.insert(model::Dispatch::new_empty(dispatch_name));
            analysis.set_model(model, None);

            retval = Some(Some(DispatchView::Manual(ManualDispatchView::new(id))));
        }

        if new_dispatch_auto == Some(true) {
            // Create new plan and set it to current
            let mut model = analysis.model().clone();
            let plan_name = format!("Plan {}", model.plans.next_id()+1);
            let id = model.plans.insert(model::PlanSpec::new_empty(plan_name));
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
