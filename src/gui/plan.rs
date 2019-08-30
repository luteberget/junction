use std::collections::HashMap;
use const_cstr::*;
use backend_glfw::imgui::*;
use std::ffi::CString;

use crate::app::*;
use crate::document::model::*;
use crate::document::*;
use crate::document::viewmodel::ViewModel;
use crate::gui::widgets;
use crate::config::*;

pub fn plan_view(app :&mut App) {
    if let Some(DispatchView::Auto(AutoDispatchView { plan_idx, .. })) = &app.document.dispatch_view {
        edit_plan(app, *plan_idx);
    }
}
enum Action { 
    VisitMerge { source: VisitKey, target :VisitKey },
    VisitMoveBefore { source: VisitKey, target :VisitKey },
    VisitMoveToEnd { source: VisitKey, target: usize }, // Train id
    TrainVehicle { train: usize, vehicle: usize },
    NewTrain,
}


pub fn edit_plan(app :&mut App, plan_idx :ListId) {

    let mut action = None;
    let mut hovered_visit = None;
    unsafe {

        let h1 = igGetFrameHeightWithSpacing();
        let h2 = igGetFrameHeight();
        let row_height = h2 + 4.0*(h1-h2);
        let dummy_size = ImVec2 { x: 20.0, y: row_height };


        igSameLine(0.0,-1.0);
        if igButton(const_cstr!("(+) Train").as_ptr(), ImVec2::zero()) {
            action = Some(Action::NewTrain);
        }

        widgets::sep();

        let mut positions :Vec<ImVec2> = Vec::new();
        if let Some(plan) = app.document.viewmodel.model().plans.get(plan_idx) {

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
                widgets::show_text(&format!("Train {}: ", train_id));
                igSameLine(0.0,-1.0);
                igPushItemWidth(125.0);
                if let Some(new_vehicle) = select_train_combo(app.document.viewmodel.model(), vehicle_ref) {
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
                edit_visit(&app.config, &app.document.viewmodel, 
                           &mut app.document.dispatch_view, 
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
                                       app.config.color_u32(RailUIColorName::GraphCommand), 8);
                    ImDrawList_AddCircleFilled(draw_list, pos2, 8.0,
                                       app.config.color_u32(RailUIColorName::GraphCommand), 8);
                    ImDrawList_AddLine(draw_list, pos1, pos2, 
                                       app.config.color_u32(RailUIColorName::GraphTrainFront), 4.0);
                    //ImDrawList_AddLine(draw_list, elbow, pos2, 
                                       //app.config.color_u32(RailUIColorName::GraphTrainFront), 4.0);
                }
            }

            igSetCursorScreenPos(end_pos);

            if let Some(DispatchView::Auto(AutoDispatchView { plan_idx, action, .. })) = &mut app.document.dispatch_view {
                if let PlanViewAction::DragFrom(other_key, mouse_pos) = *action {
                    if !igIsMouseDown(1) {
                        *action = PlanViewAction::None;
                        if let Some(visit_key) = hovered_visit {
                            let plan_idx = *plan_idx;
                            app.document.edit_model(|m| {
                                m.plans.get_mut(plan_idx).unwrap().order.push(((other_key.train, other_key.visit), (visit_key.train, visit_key.visit), None));
                                println!("Plan is now {:#?}", m.plans.get(plan_idx).unwrap());
                                None
                            });
                        } else {
                        }
                    } else {
                        ImDrawList_AddLine(igGetForegroundDrawList(), igGetMousePos_nonUDT2().into(), mouse_pos, 
                                          igGetColorU32(ImGuiCol__ImGuiCol_Text as _, 1.0), 4.0);
                    }
                }
            }
        } else {

            // The plan did not exist! 
            // Let's close the window then.
            app.document.dispatch_view = None;
        }

    }



    match action {
        Some(Action::NewTrain) => {
            let default_train = app.document.model().vehicles.iter().next().map(|(id,_)| *id);
            app.document.edit_model(|m| {
                m.plans.get_mut(plan_idx).unwrap().trains.insert((default_train, ImShortGenList::new()));
                None
            }); },
        Some(Action::TrainVehicle { train, vehicle }) => {
            app.document.edit_model(|m| {
                if let Some(t) = m.plans.get_mut(plan_idx).unwrap().trains.get_mut(train) {
                    t.0 = Some(vehicle);
                }
                None
            });
        },
        Some(Action::VisitMerge { source, target }) => {
            if !(source.train  == target.train && source.visit == target.visit ) {
                app.document.edit_model(|m| { visit_merge(m, plan_idx, source, target); None });
            }
        },
        Some(Action::VisitMoveBefore { source, target }) => {
            if !(source.train  == target.train && source.visit == target.visit ) {
                app.document.edit_model(|m| { visit_move(m, plan_idx, source, target.train, Some(target.visit)); None });
            }
        }
        Some(Action::VisitMoveToEnd { source, target }) => {
            app.document.edit_model(|m| { visit_move(m, plan_idx, source, target, None); None });
        }
        _ => {},
    }
}

fn visit_move(m: &mut Model, plan :usize, source :VisitKey, t_train_idx: usize, idx :Option<usize>) -> Option<()> {
    let plan = m.plans.get_mut(plan)?;
    let s_train = plan.trains.get_mut(source.train)?;
    let s_visit = s_train.1.get_mut(source.visit)?;
    let new_visit = if let Some(loc_idx) = source.location {
        let data = s_visit.locs.remove(loc_idx);
        if s_visit.locs.len() == 0 { s_train.1.remove(source.visit); }
        Visit { locs: vec![data], dwell: None }
    } else {
        s_train.1.remove(source.visit)?
    };
    let t_train = plan.trains.get_mut(t_train_idx)?;
    if let Some(idx) = idx {
        t_train.1.insert_before(idx, new_visit);
    } else {
        t_train.1.insert(new_visit);
    }
    Some(())
}

fn visit_merge(m :&mut Model, plan :usize, source :VisitKey, target :VisitKey) -> Option<()> {
    let plan = m.plans.get_mut(plan)?;
    let s_train = plan.trains.get_mut(source.train)?;
    let s_visit = s_train.1.get_mut(source.visit)?;
    if let Some(loc_idx) = source.location {
        let data = s_visit.locs.remove(loc_idx);
        // no more data left, remove the visit
        if s_visit.locs.len() == 0 { s_train.1.remove(source.visit); }
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.locs.push(data);
    } else {
        let data = s_train.1.remove(source.visit)?.locs;
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.locs.extend(data);
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

fn red_button(name :*const i8) -> bool{
    unsafe {
        let c1 = ImVec4 { x: 1.0, y: 0.2,  z: 0.2, w: 0.4 };
        let c2 = ImVec4 { x: 1.0, y: 0.22, z: 0.22, w: 0.4 };
        let c3 = ImVec4 { x: 1.0, y: 0.24,  z: 0.24, w: 0.4 };
        igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        igPopStyleColor(3);
        clicked
    }
}
fn blue_button(name :*const i8) -> bool{
    unsafe {
        let c1 = ImVec4 { x: 0.22, y: 0.2,  z: 1.0, w: 0.4 };
        let c2 = ImVec4 { x: 0.24, y: 0.22, z: 1.00, w: 0.4 };
        let c3 = ImVec4 { x: 0.25, y: 0.24,  z: 1.00, w: 0.4 };
        igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
        igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
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


fn edit_visit(config :&Config, vm :&ViewModel, dispatch_view :&mut Option<DispatchView>, visit_key :VisitKey, visit :&Visit, hovered_visit :&mut Option<VisitKey>, action :&mut Option<Action>, first_visit :bool) {
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
            }

            igPopID();
        }

        igEndChild();
    }
    igPopStyleColor(1);

    if igIsItemHovered(0) && igIsMouseClicked(1, false) {
        // Maybe start drawing arrow
        if let Some(DispatchView::Auto(AutoDispatchView { action, .. })) = dispatch_view {
            if let PlanViewAction::None = action {
                *action = PlanViewAction::DragFrom(visit_key, igGetMousePos_nonUDT2().into());
            }
        }
    }

    if igIsItemHovered(0) {
        *hovered_visit = Some(visit_key);
    }


    if let Some(other_key) = drop_visitkey(key) {
        *action = Some(Action::VisitMerge { source: other_key, target: visit_key });
    }

    igPopID();
    igPopID();
}
}

fn location_marker(config :&Config, vm :&ViewModel, loc :&PlanLoc, first_visit :bool, action :&mut Option<Action>) -> Option<()> {
    unsafe {
    if good_location_marker(config, vm, loc, first_visit, action).is_err() {
        //Somethign wrong with looking up data for location marker, draw a gray '?' 
        igButton( const_cstr!("?").as_ptr() , ImVec2 { x: 0.0, y: 0.0 } );
    }
    None
    }
}

fn good_location_marker(config :&Config, vm :&ViewModel, loc :&PlanLoc, first_visit :bool, action :&mut Option<Action>) -> Result<(),()> {
    unsafe {
    let name;
    let col;
    match loc {
        Ok(Ref::Node(pt))  => {
            // assume a node here.
            let (nctype,vc) = vm.data().topology.as_ref().ok_or(())?
                .locations.get(pt).ok_or(())?;

            match nctype {
                NDType::OpenEnd => {
                    name = if vc.x > 0 { const_cstr!("->") }
                    else if vc.x < 0 { const_cstr!("<-") }
                    else if vc.y > 0 { const_cstr!("^ ") }
                    else if vc.y < 0 { const_cstr!("v ") }
                    else { return Err(()); };
                    col = if first_visit {
                        config.color_u32(RailUIColorName::CanvasTrack)
                    } else {
                        config.color_u32(RailUIColorName::CanvasRoutePath)
                    };
                },
                NDType::Sw(_)  => {
                    name = const_cstr!("Sw");
                    col = config.color_u32(RailUIColorName::GraphCommand);
                },
                NDType::Cont => { 
                    name = const_cstr!("C ");
                    col = config.color_u32(RailUIColorName::GraphTrainFront);
                }
                NDType::Crossing(_) => { 
                    name = const_cstr!("Cr");
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




