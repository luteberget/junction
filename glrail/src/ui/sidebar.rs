

use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::json::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
use crate::view::*;
use crate::command_builder::*;
use std::ptr;
use std::ffi::CString;
use const_cstr::const_cstr;

pub fn sidebar(size :ImVec2, app :&mut App) {
    unsafe {
    let v2_0 = ImVec2 { x: 0.0, y: 0.0 };
  igBeginChild(const_cstr!("Sidebar").as_ptr(), size,  false,0);

  // Start new command
    if igButton(const_cstr!("\u{f044}").as_ptr(), ImVec2 { x: 0.0, y: 0.0 })  {
        app.main_menu();
    }

    // display background job info

    igSameLine(0.0,-1.0); 
    show_text(&app.background.status_str());

  if igCollapsingHeader(const_cstr!("Vehicles").as_ptr(), 0) {
      for (i,v) in app.model.vehicles.iter_mut().enumerate() {
          igPushIDInt(i as _);
          if igCollapsingHeader(const_cstr!("Vehicle").as_ptr(), 0) {

              input_text_string(const_cstr!("Name").as_cstr(), Some(const_cstr!("Name").as_cstr()), 
                                &mut v.name, 0);
              let format = const_cstr!("%.3f").as_ptr();
              igSliderFloat(const_cstr!("Length").as_ptr(), &mut v.length as *mut _, 1.0, 1000.0, format, 1.0);
              igSliderFloat(const_cstr!("Accel").as_ptr(), &mut v.max_accel as *mut _, 0.05, 1.5, format, 1.0);
              igSliderFloat(const_cstr!("Brake").as_ptr(), &mut v.max_brake as *mut _, 0.05, 1.5, format, 1.0);
              igSliderFloat(const_cstr!("Max.vel").as_ptr(), &mut v.max_velocity as *mut _, 1.0, 200.0, format, 1.0);
          }
          igPopID();
      }
  }

  // TODO All objects list 
  //if igCollapsingHeader(const_cstr!("All objects").as_ptr(),)
  //                      0) {
  //                      //ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
  //    for (i,e) in app.model.inf.entities.iter().enumerate() {
  //        match e {
  //            Some(Entity::Track(_))  => { 
  //                let s = CString::new(format!("Track##{:?}", i)).unwrap();
  //                if igSelectable(s.as_ptr(),
  //                                app.model.view.selection == Selection::Entity(i), 0, v2_0) {
  //                    //println!("SET {}", i);
  //                    app.model.view.selection = Selection::Entity(i);
  //                }
  //            },
  //            Some(Entity::Node(p,_))   => { 
  //                let s = CString::new(format!("Node @ {}##{}", p,i)).unwrap();
  //                if igSelectable(s.as_ptr(), 
  //  
  //            app.model.view.selection == Selection::Entity(i), 0, v2_0) {
  //                    //println!("SET NODE {}", i);
  //                    app.model.view.selection = Selection::Entity(i);
  //                }
  //            },
  //            Some(Entity::Object(_,_,_)) => { 
  //                igText(const_cstr!("Object#0").as_ptr()); 
  //            },
  //            _ => {},
  //        }
  //    }
  //}

  if igCollapsingHeader(const_cstr!("Object properties").as_ptr(),
                        0) {
      // TODO FIX
                        //ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
      // let mut editaction = None;
      // let entity = app.model.selected_entity();
      // match entity {
      //     Some((id, Entity::Node(p, Node::BufferStop))) 
      //         | Some((id,Entity::Node(p, Node::Macro(_)))) => {
      //         let l_buffer = const_cstr!("Buffer stop");
      //         let l_macro = const_cstr!("Boundary");
      //         let is_buffer = if let Some(Node(_, ::Node(_,Node::BufferStop))) = entity  { true } else {false };

      //         if igBeginCombo(const_cstr!("##endtype").as_ptr(), 
      //                         if is_buffer { l_buffer.as_ptr() } else { l_macro.as_ptr() },
      //                         0) {
      //             if igSelectable(l_buffer.as_ptr(), is_buffer, 0, v2_0) && !is_buffer {
      //                 editaction = Some(InfrastructureEdit::ToggleBufferMacro(id));
      //                                       //Entity::Node(*p, Node::BufferStop)));
      //             }
      //             if igSelectable(l_macro.as_ptr(), !is_buffer, 0, v2_0) && is_buffer {
      //                 editaction = Some(InfrastructureEdit::ToggleBufferMacro(id));
      //                                       //Entity::Node(*p, Node::Macro(None))));
      //             }
      //             igEndCombo();
      //         }
      //     },
      //     Some(_) => {
      //         show_text("Other entity");
      //     },
      //     _ =>  {
      //         show_text("No entity selected.");
      //     }
      // }

      // if let Some(action) = editaction {
      //     app.integrate(AppAction::Model(ModelAction::Inf(action)));
      // }

  }


  if igCollapsingHeader(const_cstr!("Routes").as_ptr(),
                        0) {
                        //ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
      let mut hovered = None;
      match app.model.interlocking.routes {
          Derive::Ok(ref r) if r.0.len() > 0 => {
              for (i,x) in r.0.iter().enumerate() {
                  igPushIDInt(i as _);

                  if igSelectable(const_cstr!("##route").as_ptr(), false, 0, v2_0) {
                  }
                  if igIsItemHovered(0) {
                      hovered = Some(i);
                  }
                  igSameLine(0.0,-1.0);
                  show_text(&format!("entry: {:?}, exit: {:?}", x.entry, x.exit));

                  igPopID();
              }
          },
          _ => show_text("No routes available."),
      }

      app.model.view.hot_route = hovered;
  }

  if igCollapsingHeader(const_cstr!("Scenarios").as_ptr(),
                        0) {
                        //ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {

      let mut scenario_action = None;

      for (si,sc) in app.model.scenarios.iter().enumerate() {
          igPushIDInt(si as _);
          match sc {
              Scenario::Dispatch(dispatch) => {

                  let selected = SelectedScenario::Dispatch(si) == app.model.view.selected_scenario;
                  if igSelectable(const_cstr!("##dispatch").as_ptr(), selected, 0, v2_0) {
                      app.model.view.selected_scenario = if selected { SelectedScenario::None } 
                                                      else { SelectedScenario::Dispatch(si) };
                  }
                  igSameLine(0.0,-1.0);
                  show_text("Dispatch ");
                  igSameLine(0.0,-1.0);
                  show_text(&format!("{}", si));

                  if selected {
                      if let Some(cmd) = show_dispatch_command_list(&dispatch.commands) {
                          scenario_action = Some(cmd);
                      }
                  }

              },

              Scenario::Usage(movement, dispatches) => {
                  // TODO 
                  let movement_selected = if let SelectedScenario::Usage(mi,_) = &app.model.view.selected_scenario {
                      si == *mi } else { false };

                  if igSelectable(const_cstr!("##usage").as_ptr(), movement_selected, 0, v2_0) {
                      app.model.view.selected_scenario = if movement_selected { SelectedScenario::None }
                                                      else { SelectedScenario::Usage(si,None) };

                  }
                  igSameLine(0.0,-1.0);
                  show_text("Usage ");
                  igSameLine(0.0,-1.0);
                  show_text(&format!("{}", si));

                  if movement_selected {

                      // movements have
                      // 1. movements
                      //    a. movement specs
                      //       i. 
                      //       ii. list of visits
                      //    b. timing constraints
                      //       i. refs to visits in a movement (2 indices)
                      //       ii. optional time
                      // 2. derived dispatches from this spec

                      show_text("MovementSpecs");
                      for (mi,m) in movement.movements.iter().enumerate() {
                          igPushIDInt(mi as _);

                          show_text("\u{f337} Movement.");

                          // TODO will need slot maps on all of these? vehicle ref, visit ref, etc.
                          let curr_name = 
                              if let Some(v) = app.model.vehicles.get(m.vehicle_ref) {
                                  CString::new(v.name.clone()).unwrap()
                              } else { CString::new("?").unwrap() };
                          if igBeginCombo(const_cstr!("Vehicle").as_ptr(), 
                                          curr_name.as_ptr(),
                                          0) {
                              for (v_i,v) in app.model.vehicles.iter().enumerate() {
                                  igPushIDInt(v_i as _);
                                  if igSelectable(const_cstr!("##sveh").as_ptr(), m.vehicle_ref == v_i, 0, v2_0) 
                                      && m.vehicle_ref != v_i {

                                          scenario_action = Some(ScenarioEdit::SetUsageMovementVehicle(si,mi,v_i));

                                  }
                                  igSameLine(0.0,-1.0);
                                  show_text(&v.name);
                                  igPopID();
                              }
                              igEndCombo();
                          }

                          for (vi,v) in m.visits.iter().enumerate() {
                              igPushIDInt(vi as _);
                              igPushItemWidth(igGetContentRegionAvailWidth() * 0.65 - 16.0 ); 
                              // TODO encapsulate custom textfield/button widget
                              // TODO correct calculation see https://github.com/ocornut/imgui/issues/1658
                              let mut s = format!("{:?}", v);
                              igPushDisable();
                              input_text_string(const_cstr!("##x").as_cstr(), Some(const_cstr!("visit").as_cstr()),
                                &mut s, 0);
                              igPopDisable();
                              igSameLine(0.0,-1.0);


                              if igButton(const_cstr!("\u{f05b}").as_ptr(), v2_0) {
                                  let mut alb = ArgumentListBuilder::new();
                                  alb.add_usize_value("scenario",si);
                                  alb.add_usize_value("movement",mi);
                                  alb.add_usize_value("visit",vi);
                                  alb.add_id("location");
                                  alb.set_action(Box::new(|app,args| {
                                      let si = *args.get_usize("scenario").unwrap();
                                      let mi = *args.get_usize("movement").unwrap();
                                      let vi = *args.get_usize("visit").unwrap();
                                      let ni = *args.get_id("location").unwrap();
                                      app.integrate(AppAction::Model(ModelAction::Scenario(
                                                  ScenarioEdit::SetUsageMovementVisitNodes(si,mi,vi,vec![ni]))));
                                  }));
                                  app.command_builder = Some(CommandBuilder::new_screen(
                                          CommandScreen::ArgumentList(alb)));
                              }

                              igSameLine(0.0,-1.0);
                              show_text("\u{f276} Visit");
                              igSameLine(0.0,-1.0);
                              show_text(&format!("#{}", vi+1));
                              igPopItemWidth();
                              igPopID();
                          }

                          if igButton(const_cstr!("\u{f276} Add visit").as_ptr(), v2_0) {
                              scenario_action = Some(ScenarioEdit::AddUsageMovementVisit(si,mi));
                          }

                          igPopID();
                      }

                      if igButton(const_cstr!("\u{f337} Add movement").as_ptr(), v2_0) {
                          //println!(" NEW movement.");
                          scenario_action = Some(ScenarioEdit::AddUsageMovement(si));
                      }

                      show_text("Timing specs.");

                      for (ti,timing) in movement.timings.iter().enumerate() {
                          igPushIDInt(ti as _);

                          let mut am = timing.visit_a.0 as std::os::raw::c_int;
                          let mut av = timing.visit_a.1 as std::os::raw::c_int;
                          let mut bm = timing.visit_b.0 as std::os::raw::c_int;
                          let mut bv = timing.visit_b.1 as std::os::raw::c_int;
                          let mut time = timing.time.unwrap_or(-1.0);

                          if igInputInt(const_cstr!("A movement").as_ptr(), 
                                        &mut am as *mut _, 1, 1, 
                                        ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _) {
                              scenario_action = Some(ScenarioEdit::SetUsageTimingSpec(
                                      si, ti, am as _, av as _, bm as _, bv as _, 
                                      if time < 0.0 { None } else { Some(time) } )); }
                          if igInputInt(const_cstr!("A visit").as_ptr(), 
                                        &mut av as *mut _, 1, 1, 
                                        ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _) {
                              scenario_action = Some(ScenarioEdit::SetUsageTimingSpec(
                                      si, ti, am as _, av as _, bm as _, bv as _, 
                                      if time < 0.0 { None } else { Some(time) } )); }
                          if igInputInt(const_cstr!("B movement").as_ptr(), 
                                        &mut bm as *mut _, 1, 1, 
                                        ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _) {
                              scenario_action = Some(ScenarioEdit::SetUsageTimingSpec(
                                      si, ti, am as _, av as _, bm as _, bv as _, 
                                      if time < 0.0 { None } else { Some(time) } )); }
                          if igInputInt(const_cstr!("B visit").as_ptr(), 
                                        &mut bv as *mut _, 1, 1, 
                                        ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _) {
                              scenario_action = Some(ScenarioEdit::SetUsageTimingSpec(
                                      si, ti, am as _, av as _, bm as _, bv as _, 
                                      if time < 0.0 { None } else { Some(time) } )); }
                          if  igInputFloat(const_cstr!("Time").as_ptr(),
                                &mut time as *mut _, -1.0, 300.0, const_cstr!("%g").as_ptr(), 
                                ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _ ) {
                              scenario_action = Some(ScenarioEdit::SetUsageTimingSpec(
                                      si, ti, am as _, av as _, bm as _, bv as _, 
                                      if time < 0.0 { None } else { Some(time) } )); }

                          igPopID();
                      }
                      if igButton(const_cstr!("\u{f337} Add timing").as_ptr(), v2_0) {
                          //println!(" NEW movement.");
                          scenario_action = Some(ScenarioEdit::AddUsageTimingSpec(si));
                      }
                      

                      match dispatches {
                          Derive::Wait => { show_text("waiting for solver"); }
                          Derive::Err(s) => { show_text("solver error:"); igSameLine(0.0,-1.0); show_text(s); },
                          Derive::Ok(dispatches) => {
                              show_text(&format!("We have {} dispatches.", dispatches.len()));

                              let selected_dispatch = if let SelectedScenario::Usage(_,sd) = &app.model.view.selected_scenario { *sd } else { None };
                              for (di,d) in dispatches.iter().enumerate() {
                                  let selected = selected_dispatch == Some(di);
                                  if igSelectable(const_cstr!("##dispatch").as_ptr(), selected, 0, v2_0) {
                                      app.model.view.selected_scenario = if selected { SelectedScenario::Usage(si,None) } 
                                                                      else { SelectedScenario::Usage(si,Some(di)) };
                                  }
                                  igSameLine(0.0,-1.0);
                                  show_text("Dispatch ");
                                  igSameLine(0.0,-1.0);
                                  show_text(&format!("{}", si));

                                  if selected {
                                      show_dispatch_command_list(&d.commands);
                                      // read only
                                      //if let Some(cmd) = show_dispatch_command_list(&dispatch.commands) {
                                      //    scenario_action = Some(cmd);
                                      //}
                                  }
                              }
                          }
                      }

                  }
              }
          }
          igPopID();
      }

      if igButton(const_cstr!("\u{f11b} Add dispatch").as_ptr(), v2_0) {
          app.integrate(AppAction::Model(ModelAction::Scenario(
                      ScenarioEdit::NewDispatch)));
      }
      if igButton(const_cstr!("\u{f56c} Add usage (auto dispatch)").as_ptr(), v2_0) {
          app.integrate(AppAction::Model(ModelAction::Scenario(
                      ScenarioEdit::NewUsage)));
      }
      // check if any action was requested
      if let Some(action) = scenario_action {
          app.integrate(AppAction::Model(ModelAction::Scenario(action)));
      }
  }

  igEndChild();


    }

}

pub fn show_dispatch_command_list(cmds :&[(f32, Command)]) -> Option<ScenarioEdit> {
    let v2_0 = ImVec2 { x: 0.0, y: 0.0 };
    unsafe {
  show_text("Commands:");
  igColumns(3, const_cstr!("c").as_ptr(), false);
  for (i,(t,v)) in cmds.iter().enumerate() {
      igPushIDInt(i as _);
      let mut time = *t;
      igInputFloat(const_cstr!("##dtime").as_ptr(),
                    &mut time as *mut _, 0.0, 1.0, const_cstr!("%g").as_ptr(), 0);
      // TODO if edited, create action
      igNextColumn();
      match v {
          Command::Route(r) => {
              show_text(&format!("\u{f637} Route {}", r));
          },
          Command::Train(t,br) => {
              show_text(&format!("\u{f238} Train {}@{}", t, br));
          }
      }
      igNextColumn();
      if igButton(const_cstr!("\u{f55a}").as_ptr(),v2_0) {
          println!("delete");
      }
      igNextColumn();
      igPopID();
  }
  igColumns(1, ptr::null(), false);
    }

    None
}
