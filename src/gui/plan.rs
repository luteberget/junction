use std::collections::HashMap;
use const_cstr::*;
use backend_glfw::imgui::*;
use std::ffi::CString;
use nalgebra_glm as glm;

use crate::app::*;
use crate::document::model::*;
use crate::document::*;
use crate::document::analysis::Analysis;
use crate::gui::widgets;
use crate::config::*;
use crate::document::infview::{InfView, unround_coord};
use crate::gui::infrastructure::draw::box_around;
use crate::document::dispatch::DispatchOutput;

enum Action { 
    VisitDelete { key :VisitKey },
    VisitMerge { source: VisitKey, target :VisitKey },
    VisitMoveBefore { source: VisitKey, target :VisitKey },
    VisitMoveToEnd { source: VisitKey, target: usize }, // Train id
    OrderDeleteAt { key :VisitKey },
    TrainVehicle { train: usize, vehicle: usize },
    NewTrain,
    RemoveTrain { train: usize },
}

pub fn edit_plan(config :&Config, 
                 inf_canvas :Option<&widgets::Draw>,
                 inf_view :&InfView,
                 analysis :&mut Analysis, 
                 auto_dispatch :&mut AutoDispatchView) -> Option<Option<DispatchView>> {
    let plan_idx = auto_dispatch.plan_idx;

    let mut action = None;
    let mut hovered_visit = None;
    let mut new_dispatchview = None;

    unsafe {

        let h1 = igGetFrameHeightWithSpacing();
        let h2 = igGetFrameHeight();
        let row_height = h2 + 4.0*(h1-h2);
        let dummy_size = ImVec2 { x: 20.0, y: row_height };



        igSameLine(0.0,-1.0);
        if igButton(const_cstr!("\u{f0fe} \u{f239} Train").as_ptr(), ImVec2::zero()) {
            action = Some(Action::NewTrain);
        }

        igSameLine(0.0,-1.0);
        plan_dispatches(config, analysis, auto_dispatch);

        widgets::sep();

        let mut positions :Vec<ImVec2> = Vec::new();
        if let Some(plan) = analysis.model().plans.get(plan_idx) {

            let mut incoming_edges : Vec<(ListId, Vec<(bool, ListId, usize)>)> = Vec::new();
            for (train_id,(_,visits)) in plan.trains.iter() {
                incoming_edges.push((*train_id,visits.iter().map(|(v,_)| (false,*v,0usize)).collect()) );
                // mark first visit in each train for special presentation in location_marker
                let l = incoming_edges.len()-1;
                if let Some(v) = incoming_edges[l].1.get_mut(0) { v.0 = true; }
            }


            for (_,(target_train,target_visit),_) in plan.order.iter() {
                for (t, vs) in incoming_edges.iter_mut() {
                    if t == target_train {
                        for (_fst, v, n) in vs.iter_mut() {
                            if v == target_visit {
                                *n += 1;
                            }
                        }
                    }
                }
            }

            let remove_edge = |es :&mut Vec<(ListId, Vec<(bool, ListId, usize)>)>, (source_train,source_visit)| {
                for (t,vs) in es.iter_mut() {
                    if *t == source_train {
                        for (_fst, v, n) in vs.iter_mut() {
                            if *v == source_visit {
                                *n -= 1;
                            }
                        }
                    }
                }
            };

            // First draw each train as an empty row, and store the screen 
            // pos of where to start drawing visit boxes.
            for (train_id,(vehicle_ref,visits)) in plan.trains.iter() {
                igDummy(ImVec2 { x: 0.0, y: 7.0 });
                igPushIDInt(*train_id as _);

                igAlignTextToFramePadding();
                if igButton(const_cstr!("\u{f55a}").as_ptr(), ImVec2::zero()) {
                    action = Some(Action::RemoveTrain { train: *train_id });
                }
                igSameLine(0.0,-1.0);
                widgets::show_text(&format!(" \u{f239} ({}) ", train_id));
                igSameLine(0.0,-1.0);
                igPushItemWidth(125.0);
                if let Some(new_vehicle) = select_train_combo(analysis.model(), vehicle_ref) {
                    action = Some(Action::TrainVehicle { train: *train_id, vehicle: new_vehicle });
                }
                igPopItemWidth();

                // If there are no visits, draw a yellow suggested visit box 
                if visits.iter().next().is_none() {
                    //igSameLine(125.0 + igGetFrameHeight(),-1.0);
                    igSameLine(0.0,-1.0);

                    yellow_button(const_cstr!("?").as_ptr());
                }

                igSameLine(0.0,-1.0);
                let pos :ImVec2 = igGetCursorScreenPos_nonUDT2().into();
                positions.push(pos + ImVec2 { x: 0.0, y: -5.0 } );
                igPopID();
                igNewLine();
                igDummy(ImVec2 { x: 0.0, y: 7.0 });
                widgets::sep();
            }

            let end_pos :ImVec2 = igGetCursorScreenPos_nonUDT2().into();
            let mut visit_pos :HashMap<VisitKey, ImVec2> = HashMap::new();

            // Find a train whose leftmost visit has no incoming constraints
            while let Some((train_idx, (train_id,visits))) = incoming_edges.iter_mut()
                .enumerate().find(|(_,(_,vs))| vs.get(0).map(|x| x.2) == Some(0)) {

                let (first_visit,visit_id,zero_incoming) = visits.remove(0);
                assert_eq!(zero_incoming, 0);

                // lookup the visit info in the plan
                let visit = plan.trains.get(*train_id).unwrap().1.get(visit_id).unwrap();

                let train_id = *train_id;

                // Then check constraints here, decrement incoming on targets.
                for ((strain,svisit),(ttrain,tvisit),_) in plan.order.iter() {
                    if *strain == train_id && *svisit == visit_id {
                        remove_edge(&mut incoming_edges, (*ttrain,*tvisit));
                        // if this is the case, we need to advance the cursor to be to the right
                        let other_train_idx = incoming_edges.iter().position(|(t,_)| t == ttrain).unwrap();

                        let this_x = positions[train_idx].x;
                        let other_x = &mut positions[other_train_idx].x;
                        *other_x = other_x.max(this_x + 32.0);
                    }
                }

                igSetCursorScreenPos(positions[train_idx]);
                let vkey = VisitKey { train: train_id, visit: visit_id, location: None }; 
                visit_pos.insert(vkey, positions[train_idx] +
                                 ImVec2 { x: 2.0*dummy_size.x, y: 0.5* dummy_size.y });

                // Draw the visit here.
                edit_visit(config, analysis, auto_dispatch, 
                           vkey, visit, &mut hovered_visit, &mut action, first_visit);
                igSameLine(0.0,-1.0);

                let new_pos =  igGetCursorScreenPos_nonUDT2().into();
                positions[train_idx] = new_pos;
            }


            for (pos,(train_id,_)) in positions.into_iter().zip(incoming_edges.iter()) {
                igSetCursorScreenPos(pos);
                igPushIDInt(123123 as _);
                igDummy(dummy_size + ImVec2 { x: 1.0*32.0, y: 0.0 });
                let key = const_cstr!("VISIT").as_ptr();
                if let Some(other_key) = drop_visitkey(key) {
                    action = Some(Action::VisitMoveToEnd { source: other_key, target: *train_id });
                }
                igPopID();
            }

            // Draw constraints
            let draw_list = igGetWindowDrawList();
            for ((strain,svisit),(ttrain,tvisit),_) in plan.order.iter() {
                let pos1 = visit_pos.get(&VisitKey { train: *strain, visit: *svisit, location: None });
                let pos2 = visit_pos.get(&VisitKey { train: *ttrain, visit: *tvisit, location: None });
                if let (Some(pos1),Some(pos2)) = (pos1.cloned(),pos2.cloned()) {
                    let elbow = ImVec2 { x: pos1.x, y: pos2.y };
                    ImDrawList_AddCircleFilled(draw_list, pos1, 8.0,
                                       config.color_u32(RailUIColorName::GraphCommandRoute), 8);
                    ImDrawList_AddCircleFilled(draw_list, pos2, 8.0,
                                       config.color_u32(RailUIColorName::GraphCommandRoute), 8);
                    ImDrawList_AddLine(draw_list, pos1, pos2, 
                                       config.color_u32(RailUIColorName::GraphTrainFront), 4.0);
                    //ImDrawList_AddLine(draw_list, elbow, pos2, 
                                       //app.config.color_u32(RailUIColorName::GraphTrainFront), 4.0);
                }
            }

            igSetCursorScreenPos(end_pos);

            // Draw hovered visits/location in infrastructure view
            draw_hovered_inf(config, analysis.model(), plan_idx, &hovered_visit, inf_canvas, inf_view);


            if let PlanViewAction::DragFrom(other_key, mouse_pos) = auto_dispatch.action {
                if igIsMouseClicked(0, false) || igIsMouseClicked(1, false) {
                    auto_dispatch.action = PlanViewAction::None;
                    if let Some(visit_key) = hovered_visit {
                        if !(other_key.train == visit_key.train && other_key.visit == visit_key.visit) {
                            analysis.edit_model(|m| {
                                m.plans.get_mut(plan_idx).unwrap().order
                                    .push(((other_key.train, other_key.visit), 
                                           (visit_key.train, visit_key.visit), None));
                                //println!("Plan is now {:#?}", m.plans.get(plan_idx).unwrap());
                                None
                            });
                        }
                    } else {
                    }
                } else {
                    ImDrawList_AddLine(igGetForegroundDrawList(), igGetMousePos_nonUDT2().into(), mouse_pos, 
                                      igGetColorU32(ImGuiCol__ImGuiCol_Text as _, 1.0), 4.0);
                }
            }
        } else {

            // The plan did not exist! 
            // Let's close the window then.

            // app.document.dispatch_view = None;
            // TODO by returning Err?
            new_dispatchview = Some(None);
        }

    }



    match action {
        Some(Action::NewTrain) => {
            let default_train = analysis.model().vehicles.iter().next().map(|(id,_)| *id);
            analysis.edit_model(|m| {
                m.plans.get_mut(plan_idx).unwrap().trains.insert((default_train, ImShortGenList::new()));
                None
            }); },
        Some(Action::RemoveTrain { train }) => {
            let default_train = analysis.model().vehicles.iter().next().map(|(id,_)| *id);
            analysis.edit_model(|m| {
                m.plans.get_mut(plan_idx).unwrap().trains.remove(train);

                // Remove all constraints referencing the train
                let plan = m.plans.get_mut(plan_idx)?;
                remove_ordering_train(plan, train);

                None
            }); },
        Some(Action::TrainVehicle { train, vehicle }) => {
            analysis.edit_model(|m| {
                if let Some(t) = m.plans.get_mut(plan_idx).unwrap().trains.get_mut(train) {
                    t.0 = Some(vehicle);
                }
                None
            });
        },
        Some(Action::VisitMerge { source, target }) => {
            if !(source.train  == target.train && source.visit == target.visit ) {
                analysis.edit_model(|m| { visit_merge(m, plan_idx, source, target); None });
            }
        },
        Some(Action::VisitMoveBefore { source, target }) => {
            if !(source.train  == target.train && source.visit == target.visit ) {
                analysis.edit_model(|m| { visit_move(m, plan_idx, source, target.train, Some(target.visit)); None });
            }
        }
        Some(Action::VisitMoveToEnd { source, target }) => {
            analysis.edit_model(|m| { visit_move(m, plan_idx, source, target, None); None });
        }
        Some(Action::OrderDeleteAt { key }) => {
            analysis.edit_model(|m| {
                let plan = m.plans.get_mut(plan_idx)?;
                remove_ordering_at(plan, key.train, key.visit);
                None
            });
        },
        Some(Action::VisitDelete { key }) => {
            analysis.edit_model(|m| {
                let plan = m.plans.get_mut(plan_idx)?;
                let (_,train) = plan.trains.get_mut(key.train)?;
                let visit = train.get_mut(key.visit)?;
                let deleted_visit = if let Some(loc_idx) = key.location {
                    visit.locs.remove(loc_idx);
                    if visit.locs.len() == 0 { train.remove(key.visit); true } else { false }
                } else {
                    train.remove(key.visit)?; true
                };

                if deleted_visit {
                    remove_ordering_at(plan, key.train, key.visit);
                }

                None
            });
        }
        _ => {},
    }

    new_dispatchview
}

pub fn planning_icon(config :&Config, analysis :&Analysis, generation :usize, dispatches :&Vec<DispatchOutput>) {
    unsafe {
    if generation == *analysis.generation() {
        if dispatches.len() > 0 {
            // Planning was successful
            igPushStyleColorU32(ImGuiCol__ImGuiCol_Text as _, 
                                config.color_u32(RailUIColorName::CanvasSignalProceed));
            widgets::show_text("\u{f00c}");
            igPopStyleColor(1);
        } else {
            // Planning failed
            igPushStyleColorU32(ImGuiCol__ImGuiCol_Text as _, 
                                config.color_u32(RailUIColorName::CanvasSignalStop));
            widgets::show_text("\u{f00d}");
            igPopStyleColor(1);
        }
    } else {
        // Planning still running 
        igPushStyleColorU32(ImGuiCol__ImGuiCol_Text as _, 
                            config.color_u32(RailUIColorName::CanvasTrackDrawing));
        widgets::show_text("\u{f110}");
        igPopStyleColor(1);
    }
    }
}

fn plan_dispatches(config :&Config, analysis :&Analysis, adv :&mut AutoDispatchView)  {
    unsafe {
        if let Some(Some((generation,dispatches))) = analysis.data().plandispatches.get(adv.plan_idx) {
            planning_icon(config,analysis,*generation,dispatches);
            igSameLine(0.0,-1.0);

            let dispatch_idx = if let Some(ManualDispatchView { dispatch_idx, .. }) = &adv.dispatch {
                Some(*dispatch_idx) } else { None };
            let dispatch_name = if let Some(dispatch_idx) = dispatch_idx {
                CString::new(format!("Dispatch {}", dispatch_idx)).unwrap()
            } else { CString::new(format!("None")).unwrap() };

            if igBeginCombo(const_cstr!("##chtr").as_ptr(), dispatch_name.as_ptr(), 0) {
                if igSelectable(const_cstr!("None").as_ptr(), dispatch_idx.is_none(), 0 as _, ImVec2::zero()) {

                    adv.dispatch = None;
                }

                for (di,d) in dispatches.iter().enumerate() {
                    igPushIDInt(di as _);
                    if igSelectable(const_cstr!("##asdf").as_ptr(), 
                                 dispatch_idx == Some(di), 
                                 0 as _ , ImVec2::zero()) {
                        adv.dispatch = Some(ManualDispatchView::new(di));
                    }

                    igSameLine(0.0,-1.0);
                    widgets::show_text(&format!("Dispatch {}", di));
                    igPopID();
                }

                igEndCombo();
            }
        }
    }
}

fn visit_move(m: &mut Model, plan :usize, source :VisitKey, t_train_idx: usize, idx :Option<usize>) -> Option<()> {
    let plan = m.plans.get_mut(plan)?;
    let s_train = plan.trains.get_mut(source.train)?;
    let s_visit = s_train.1.get_mut(source.visit)?;
    let (new_visit,deleted_visit) = if let Some(loc_idx) = source.location {
        let data = s_visit.locs.remove(loc_idx);
        let deleted = if s_visit.locs.len() == 0 { s_train.1.remove(source.visit); true } else { false };
        (Visit { locs: vec![data], dwell: None },deleted)
    } else {
        (s_train.1.remove(source.visit)?, true)
    };
    let t_train = plan.trains.get_mut(t_train_idx)?;
    let new_idx = if let Some(idx) = idx {
        t_train.1.insert_before(idx, new_visit)
    } else {
        t_train.1.insert(new_visit)
    };
    if deleted_visit {
        rename_train_visit(plan, source.train, source.visit, t_train_idx, new_idx);
    }
    Some(())
}

fn rename_train_visit(plan :&mut PlanSpec, train :usize, visit :usize, new_train :usize, new_visit :usize) {
    for (a,b,_) in plan.order.iter_mut() {
        if a.0 == train && a.1 == visit {
            a.0 = new_train;
            a.1 = new_visit;
        }
        if b.0 == train && b.1 == visit {
            b.0 = new_train;
            b.1 = new_visit;
        }
    }
}

fn remove_ordering_train(plan :&mut PlanSpec, train :usize) {
    plan.order.retain(|(a,b,_)| {
        let a = a.0 == train;
        let b = b.0 == train;
        !a && !b
    });
}

fn remove_ordering_at(plan :&mut PlanSpec, train :usize, visit :usize) {
    plan.order.retain(|(a,b,_)| {
        let a = a.0 == train && a.1 == visit;
        let b = b.0 == train && b.1 == visit;
        !a && !b
    });
}

fn visit_merge(m :&mut Model, plan :usize, source :VisitKey, target :VisitKey) -> Option<()> {
    let plan = m.plans.get_mut(plan)?;
    let s_train = plan.trains.get_mut(source.train)?;
    let s_visit = s_train.1.get_mut(source.visit)?;
    let deleted_visit = if let Some(loc_idx) = source.location {
        let data = s_visit.locs.remove(loc_idx);
        // no more data left, remove the visit
        let delete = if s_visit.locs.len() == 0 { s_train.1.remove(source.visit); true } else { false };
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.locs.push(data);
        delete
    } else {
        let data = s_train.1.remove(source.visit)?.locs;
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.locs.extend(data);
        true
    };
    if deleted_visit {
        remove_ordering_at(plan, source.train, source.visit);
    }
    Some(())
}

    //VisitMerge { source: VisitKey, target :VisitKey },
    //VisitMoveBefore { source: VisitKey, target :VisitKey },
    //VisitMoveToEnd { source: VisitKey, target: usize }, // Train id
    //TrainVehicle { train: usize, vehicle: usize },
    //NewTrain,

pub fn select_train_combo(model :&Model, current_id :&Option<usize>) -> Option<ListId> {
    let mut v = None;
    unsafe {
        let mut v_name = None;
        if let Some(id) = current_id { 
            if let Some(v) = model.vehicles.get(*id) {
                v_name = Some(CString::new(v.name.clone()).unwrap());
            }
        }

        let v_name = v_name.unwrap_or_else(|| CString::new("?").unwrap());

        if igBeginCombo(const_cstr!("##chtr").as_ptr(), v_name.as_ptr(), 0) {
            v = select_train(model, current_id);
            igEndCombo();
        }
    }
    v
}

pub fn select_train(model :&Model, current_id :&Option<usize>) -> Option<ListId> {
    let mut retval = None;
    let mut any = false;
    unsafe {
        for (i,v) in model.vehicles.iter() {
            any = true;
            igPushIDInt(*i as _);
            if igSelectable(const_cstr!("##vh").as_ptr(), Some(*i) == *current_id, 0 as _, ImVec2::zero()) {
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

fn drop_visitkey(key :*const i8) -> Option<VisitKey> {
    unsafe {
    if igBeginDragDropTarget() {
        let payload = igAcceptDragDropPayload(key, 0 as _) as *mut ImGuiPayload;
        if payload != std::ptr::null_mut() {
            let k :VisitKey = *((*payload).Data as *const VisitKey);
            return Some(k);
        }
        igEndDragDropTarget();
    }
    None
    }
}


fn edit_visit(config :&Config, vm :&Analysis, auto_dispatch :&mut AutoDispatchView, 
              visit_key :VisitKey, visit :&Visit, hovered_visit :&mut Option<VisitKey>, 
              action :&mut Option<Action>, first_visit :bool) {
unsafe {
    let key = const_cstr!("VISIT").as_ptr();
    igPushIDInt(visit_key.train as _); 
    igPushIDInt(visit_key.visit as _); 

    let h1 = igGetFrameHeightWithSpacing();
    let h2 = igGetFrameHeight();
    let row_height = h2 + 4.0*(h1-h2);
    let dummy_size = ImVec2 { x: 20.0, y: row_height };

    //for (visit_id, visit) in visits.iter() {
    //    let mut visit_key = VisitKey { train: *train_id, visit: *visit_id, location: None };

    igDummy(dummy_size);
    if let Some(other_key) = drop_visitkey(key) {
        *action = Some(Action::VisitMoveBefore { source: other_key, target: visit_key });
    }
    igSameLine(0.0,-1.0);
    let visit_width = 32.0*(1.0 + visit.locs.len() as f32 + 
                                if visit.dwell.is_some() { 1.0 } else { 0.0 });
    igPushStyleColorU32(ImGuiCol__ImGuiCol_ChildBg as _,
                          igGetColorU32(ImGuiCol__ImGuiCol_Button as _, 1.0));
    let mut hovered_location = None;
    if igBeginChild(const_cstr!("##vfrm").as_ptr(), ImVec2 { x: visit_width, y: row_height}, true, 0 as _) {
        if igBeginDragDropSource(0 as _) {

            let mut visit_key = visit_key;
            igSetDragDropPayload(key, 
                 &mut visit_key as *mut VisitKey as *mut std::ffi::c_void, 
                                 std::mem::size_of::<VisitKey>(), 0 as _);

            igAlignTextToFramePadding();
            widgets::show_text(&format!("Move"));
            for (loc_id, loc) in visit.locs.iter().enumerate() {
                igPushIDInt(loc_id as _);
                igSameLine(0.0,-1.0);
                location_marker(config, vm, loc, first_visit, action);
                igPopID();
            }

            igEndDragDropSource();
        }


        for (loc_id,loc) in visit.locs.iter().enumerate() {
            igPushIDInt(loc_id as _);
            let mut visit_key = VisitKey { location: Some(loc_id), .. visit_key };
            location_marker(config,vm,loc,first_visit,action);
            igSameLine(0.0,-1.0);
            if igBeginDragDropSource(0 as _) {
                igSetDragDropPayload(key, 
                                     &mut visit_key as *mut VisitKey as *mut std::ffi::c_void,
                                     std::mem::size_of::<VisitKey>(), 0 as _);
                igAlignTextToFramePadding();
                widgets::show_text(&format!("Move"));
                igSameLine(0.0,-1.0);
                location_marker(config,vm,loc,first_visit,action);
                igEndDragDropSource();
            } else if igIsItemHovered(0) {
                igBeginTooltip();
                widgets::show_text(&format!("Visit {} {:?}", visit_key.visit, visit));
                igEndTooltip();
                hovered_location = Some(loc_id);
            }

            igPopID();
        }

        igEndChild();
    }
    igPopStyleColor(1);

    if igIsItemHovered(0) && igIsMouseClicked(1, false) {
        auto_dispatch.action = PlanViewAction::Menu(VisitKey { location: hovered_location, .. visit_key }, 
                                                    igGetMousePos_nonUDT2().into());
        igOpenPopup(const_cstr!("pctx").as_ptr());
    }

    if igBeginPopup(const_cstr!("pctx").as_ptr(), 0 as _) {
        match auto_dispatch.action {
            PlanViewAction::Menu(key, pos) => {
                if igSelectable(const_cstr!("\u{f0dc} Add ordering constraint...").as_ptr(), false, 0 as _, ImVec2::zero()) {
                    auto_dispatch.action = PlanViewAction::DragFrom(key,pos);
                }

                widgets::sep();
                
                if key.location.is_some() {
                    if igSelectable(const_cstr!("\u{f55a} Remove location").as_ptr(), false, 0 as _, ImVec2::zero()) {
                        *action = Some(Action::VisitDelete { key });
                    }
                }
                if igSelectable(const_cstr!("\u{f55a} Remove visit").as_ptr(), false, 0 as _, ImVec2::zero()) {
                    *action = Some(Action::VisitDelete { key: VisitKey { location: None, .. key } } );
                }
                if igSelectable(const_cstr!("\u{f55a} Remove ordering constraints").as_ptr(), false, 0 as _, ImVec2::zero()) {
                    *action = Some(Action::OrderDeleteAt { key });
                }
            },
            _ => {
                widgets::show_text("No visit selected.");
            }
        }
        igEndPopup();
    }

    if igIsItemHovered(0) {
        *hovered_visit = Some(VisitKey { location: hovered_location, .. visit_key});
    }


    if let Some(other_key) = drop_visitkey(key) {
        *action = Some(Action::VisitMerge { source: other_key, target: visit_key });
    }

    igPopID();
    igPopID();
}
}

fn location_marker(config :&Config, vm :&Analysis, loc :&PlanLoc, first_visit :bool, action :&mut Option<Action>) -> Option<()> {
    unsafe {
    if good_location_marker(config, vm, loc, first_visit, action).is_err() {
        //Somethign wrong with looking up data for location marker, draw a gray '?' 
        igButton( const_cstr!("?").as_ptr() , ImVec2 { x: 0.0, y: 0.0 } );
    }
    None
    }
}

fn good_location_marker(config :&Config, vm :&Analysis, loc :&PlanLoc, first_visit :bool, action :&mut Option<Action>) -> Result<(),()> {
    unsafe {
    let name;
    let col;
    match loc {
        Ok(Ref::Node(pt))  => {
            // assume a node here.
            let (nctype,vc) = vm.data().topology.as_ref().ok_or(())?.1
                .locations.get(pt).ok_or(())?;

            match nctype {
                NDType::OpenEnd => {
                    name = if (vc.x > 0 && first_visit) || (vc.x < 0 && !first_visit) { 
                        const_cstr!("\u{f061}") }
                    else if (vc.x < 0 && first_visit) || (vc.x > 0 && !first_visit) { 
                        const_cstr!("\u{f060}") }
                    else if (vc.y > 0 && first_visit) || (vc.y < 0 && !first_visit) { 
                        const_cstr!("\u{f062}") }
                    else if (vc.y < 0 && first_visit) || (vc.y > 0 && !first_visit) { 
                        const_cstr!("\u{f063}") }
                    else { return Err(()); };
                    col = if first_visit {
                        config.color_u32(RailUIColorName::CanvasTrack)
                    } else {
                        config.color_u32(RailUIColorName::CanvasRoutePath)
                    };
                },
                NDType::Sw(_)  => {
                    name = const_cstr!("\u{f074}");
                    col = config.color_u32(RailUIColorName::GraphCommandRoute);
                },
                NDType::Cont => { 
                    name = const_cstr!("\u{f337}");
                    col = config.color_u32(RailUIColorName::GraphTrainFront);
                }
                NDType::Crossing(_) => { 
                    name = const_cstr!("\u{f074}");
                    col = config.color_u32(RailUIColorName::GraphBlockReserved);
                },
                NDType::Err | NDType::BufferStop => { return Err(()); }
            };

        },
        Ok(Ref::LineSeg(_,_)) => {
            name = const_cstr!("--");
            col = config.color_u32(RailUIColorName::CanvasTrackDrawing);
        },
        Ok(Ref::Object(_)) => {
            // TODO check object type or just allow signals?
            name = const_cstr!("-O");
            col = config.color_u32(RailUIColorName::CanvasSignalStop);
        }
        Err(ptc) =>  {
            return Err(());
        }
    };
    igPushStyleColorU32(ImGuiCol__ImGuiCol_Button as _, col);
    igButton(name.as_ptr(), ImVec2::zero());
    igPopStyleColor(1);
    Ok(())
    }
}



fn draw_hovered_inf(config :&Config, model :&Model, plan_idx :usize, hovered_visit :&Option<VisitKey>, 
                    inf_canvas :Option<&widgets::Draw>, inf_view :&InfView) -> Option<()> {
    let visit_key = hovered_visit.as_ref()?;
    let draw = inf_canvas?;
    let visit = model.plans.get(plan_idx)?
        .trains.get(visit_key.train)?
        .1.get(visit_key.visit)?;

    for (loc_id,loc) in visit.locs.iter().enumerate() {
        if visit_key.location.is_none() || visit_key.location.unwrap() == loc_id {
            let pt = match loc {
                Ok(Ref::Node(pt)) => glm::vec2(pt.x as f32, pt.y as f32),
                Ok(Ref::Object(pta)) => unround_coord(*pta),
                Ok(Ref::LineSeg(a,b)) => glm::vec2(a.x as f32, a.y as f32),
                Err(p) => *p,
            };
            box_around(config, draw, inf_view, pt);
        }
    }
    Some(())
}

