use const_cstr::*;
use backend_glfw::imgui::*;
use std::ffi::CString;

use crate::app::*;
use crate::document::model::*;
use crate::document::*;
use crate::gui::widgets;

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

        //igDummy(ImVec2 { x: 0.0, y: 7.0 } );

        //let toposort = get_toposort(app, plan_idx);

        let mut positions :Vec<ImVec2> = Vec::new();

        let key = const_cstr!("VISIT").as_ptr();
        if let Some(plan) = app.document.viewmodel.model().plans.get(plan_idx) {
            for (train_id,(vehicle_ref,visits)) in plan.trains.iter() {
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

                positions.push(igGetCursorPos_nonUDT2().into());


                // write each visit.
                // TODO do this afterwards, using sorting information from ordering constraints
                for (visit_id, visit) in visits.iter() {
                    let mut visit_key = VisitKey { train: *train_id, visit: *visit_id, location: None };
                    igPushIDInt(*visit_id as _);
                    igSameLine(0.0,-1.0);
                    igDummy(dummy_size);
                    if let Some(other_key) = drop_visitkey(key) {
                        action = Some(Action::VisitMoveBefore { source: other_key,
                            target: VisitKey { train: *train_id, visit: *visit_id, location: None }});

                    }

                    igSameLine(0.0,-1.0);
                    let visit_width = 32.0*(1.0 + visit.loc.len() as f32 + 
                                                if visit.dwell.is_some() { 1.0 } else { 0.0 });
                    igPushStyleColorU32(ImGuiCol__ImGuiCol_ChildBg,
                                          igGetColorU32(ImGuiCol__ImGuiCol_Button, 1.0));
                    if igBeginChild(const_cstr!("##vfrm").as_ptr(), ImVec2 { x: visit_width, y: row_height}, true, 0 as _) {
                        if igBeginDragDropSource(0 as _) {

                            igSetDragDropPayload(key, 
                                 &mut visit_key as *mut VisitKey as *mut std::ffi::c_void, 
                                                 std::mem::size_of::<VisitKey>(), 0 as _);

                            igAlignTextToFramePadding();
                            widgets::show_text(&format!("Move"));
                            for (loc_id, loc) in visit.loc.iter().enumerate() {
                                igPushIDInt(loc_id as _);
                                igSameLine(0.0,-1.0);
                                red_button(const_cstr!("Nd").as_ptr());
                                igPopID();
                            }

                            igEndDragDropSource();
                        }


                        for (loc_id,loc) in visit.loc.iter().enumerate() {
                            igPushIDInt(loc_id as _);
                            let mut visit_key = VisitKey { location: Some(loc_id), .. visit_key };
                            red_button(const_cstr!("Nd").as_ptr());
                            igSameLine(0.0,-1.0);
                            if igBeginDragDropSource(0 as _) {
                                igSetDragDropPayload(key, 
                                                     &mut visit_key as *mut VisitKey as *mut std::ffi::c_void,
                                                     std::mem::size_of::<VisitKey>(), 0 as _);
                                igAlignTextToFramePadding();
                                widgets::show_text(&format!("Move"));
                                igSameLine(0.0,-1.0);
                                red_button(const_cstr!("Nd").as_ptr());
                                igEndDragDropSource();
                            } else if igIsItemHovered(0) {
                                igBeginTooltip();
                                widgets::show_text(&format!("Visit {} {:?}", visit_id, visit));
                                igEndTooltip();
                            }

                            igPopID();
                        }

                        igEndChild();
                    }
                    igPopStyleColor(1);

                    if igIsItemHovered(0) && igIsMouseClicked(1, false) {
                        // Maybe start drawing arrow
                        if let Some(DispatchView::Auto(AutoDispatchView { action, .. })) = &mut app.document.dispatch_view {
                            if let PlanViewAction::None = action {
                                *action = PlanViewAction::DragFrom(visit_key, igGetMousePos_nonUDT2().into());
                            }
                        }
                    }

                    if igIsItemHovered(0) {
                        hovered_visit = Some(visit_key);
                    }


                    if let Some(other_key) = drop_visitkey(key) {
                        action = Some(Action::VisitMerge { source: other_key, target: visit_key });
                    }


                    igPopID();
                }

                igPushIDInt(123123 as _);
                igSameLine(0.0,-1.0);
                igDummy(dummy_size + ImVec2 { x: 2.0*32.0, y: 0.0 });
                if let Some(other_key) = drop_visitkey(key) {
                    action = Some(Action::VisitMoveToEnd { source: other_key, target: *train_id });
                }
                igPopID();

                widgets::sep();
                igPopID();
            }

            if let Some(DispatchView::Auto(AutoDispatchView { action, .. })) = &mut app.document.dispatch_view {
                if let PlanViewAction::DragFrom(other_key, mouse_pos) = *action {
                    if !igIsMouseDown(1) {
                        *action = PlanViewAction::None;
                        if let Some(visit_key) = hovered_visit {
                            println!("ADD CONSTRAINT {:?} {:?}", other_key, visit_key);
                        } else {
                            println!("Dragged outside");
                        }
                    } else {
                        ImDrawList_AddLine(igGetForegroundDrawList(), igGetMousePos_nonUDT2().into(), mouse_pos, 
                                          igGetColorU32(ImGuiCol__ImGuiCol_Text, 1.0), 4.0);
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
        let data = s_visit.loc.remove(loc_idx);
        if s_visit.loc.len() == 0 { s_train.1.remove(source.visit); }
        Visit { loc: vec![data], dwell: None }
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
        let data = s_visit.loc.remove(loc_idx);
        // no more data left, remove the visit
        if s_visit.loc.len() == 0 { s_train.1.remove(source.visit); }
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.loc.push(data);
    } else {
        let data = s_train.1.remove(source.visit)?.loc;
        let t_train = plan.trains.get_mut(target.train)?;
        let t_visit = t_train.1.get_mut(target.visit)?;
        t_visit.loc.extend(data);
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

fn draw_location_buttons()  {
}
