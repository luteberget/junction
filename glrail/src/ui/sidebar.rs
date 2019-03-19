

use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::json::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
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

  if igCollapsingHeader(const_cstr!("All objects").as_ptr(),
                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
      for (i,e) in app.model.inf.entities.iter().enumerate() {
          match e {
              Some(Entity::Track(_))  => { 
                  let s = CString::new(format!("Track##{}", i)).unwrap();
                  if igSelectable(s.as_ptr(),
                                  app.model.view.selection == Selection::Entity(i), 0, v2_0) {
                      //println!("SET {}", i);
                      app.model.view.selection = Selection::Entity(i);
                  }
              },
              Some(Entity::Node(p,_))   => { 
                  let s = CString::new(format!("Node @ {}##{}", p,i)).unwrap();
                  if igSelectable(s.as_ptr(), 
    
              app.model.view.selection == Selection::Entity(i), 0, v2_0) {
                      //println!("SET NODE {}", i);
                      app.model.view.selection = Selection::Entity(i);
                  }
              },
              Some(Entity::Object(_,_,_)) => { 
                  igText(const_cstr!("Object#0").as_ptr()); 
              },
              _ => {},
          }
      }
  }

  if igCollapsingHeader(const_cstr!("Object properties").as_ptr(),
                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
      let mut editaction = None;
      let entity = app.model.selected_entity();
      match entity {
          Some((id, Entity::Node(p, Node::BufferStop))) 
              | Some((id,Entity::Node(p, Node::Macro(_)))) => {
              let l_buffer = const_cstr!("Buffer stop");
              let l_macro = const_cstr!("Boundary");
              let is_buffer = if let Some((_, Entity::Node(_,Node::BufferStop))) = entity  { true } else {false };

              if igBeginCombo(const_cstr!("##endtype").as_ptr(), 
                              if is_buffer { l_buffer.as_ptr() } else { l_macro.as_ptr() },
                              0) {
                  if igSelectable(l_buffer.as_ptr(), is_buffer, 0, v2_0) && !is_buffer {
                      editaction = Some(InfrastructureEdit::UpdateEntity(id,
                                            Entity::Node(*p, Node::BufferStop)));
                  }
                  if igSelectable(l_macro.as_ptr(), !is_buffer, 0, v2_0) && is_buffer {
                      editaction = Some(InfrastructureEdit::UpdateEntity(id,
                                            Entity::Node(*p, Node::Macro(None))));
                  }
                  igEndCombo();
              }
          },
          Some(_) => {
              show_text("Other entity");
          },
          _ =>  {
              show_text("No entity selected.");
          }
      }

      if let Some(action) = editaction {
          app.integrate(AppAction::Model(ModelAction::Inf(action)));
      }

  }


  if igCollapsingHeader(const_cstr!("Routes").as_ptr(),
                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
      let mut hovered = None;
      match app.model.interlocking.routes {
          Derive::Ok(ref r) if r.len() > 0 => {
              for (i,x) in r.iter().enumerate() {
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
  ////println!("hot route: {:?}", app.model.view.hot_route);
  // if igCollapsingHeader(const_cstr!("Scenarios").as_ptr(),
  //                       ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
  //     //for r in &app.model.scenarios {

  //     //}
  // }

  //if igCollapsingHeader(const_cstr!("User data editor").as_ptr(),
  //                      ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
  //    json_editor(&json_types, user_data.as_object_mut().unwrap(), &mut open_object);
  //}

  if igCollapsingHeader(const_cstr!("Scenarios").as_ptr(),
                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {

      for (si,sc) in app.model.scenarios.iter_mut().enumerate() {
          igPushIDInt(si as _);
          match sc {
              Scenario::Dispatch(dispatch) => {

                  let selected = Some(si) == app.model.view.selected_dispatch;
                  //println!("selected {:?} {:?}", app.model.view.selected_dispatch, selected);
                  if igSelectable(const_cstr!("##dispatch").as_ptr(), selected, 0, v2_0) {
                      app.model.view.selected_dispatch = if selected { None } else { Some(si) };
                      println!("clicked");
                  }
                  igSameLine(0.0,-1.0);
                  show_text("Dispatch ");
                  igSameLine(0.0,-1.0);
                  show_text(&format!("{}", si));

                  if selected {
                      show_text("Commands:");

                      igColumns(3, const_cstr!("c").as_ptr(), false);

                      for (i,(t,v)) in dispatch.commands.iter_mut().enumerate() {
                          igPushIDInt(i as _);

                          igInputFloat(const_cstr!("##dtime").as_ptr(),
                                        t as *mut _, 0.0, 1.0, const_cstr!("%g").as_ptr(), 0);

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

              },

              Scenario::Movement(movement, dispatches) => {
                  // TODO 

                  //show_text("movement1");

                  //match dispatches {
                  //    Derive::Ok(dispatches) => {
                  //        for (di,dispatch) in dispatches.iter().enumerate() {
                  //            igPushIDInt(di as _);
                  //            show_text("mdispatch1");
                  //            igPopID();
                  //        }
                  //    },
                  //    Derive::Err(msg) => {
                  //        show_text("error: "); igSameLine(0.0, -1.0); show_text(&msg);
                  //    }
                  //    Derive::Wait => {
                  //        show_text("calulating...");
                  //    }
                  //}
              }
          }
          igPopID();
      }

      if igButton(const_cstr!("\u{f11b} Add dispatch").as_ptr(), v2_0) {
          app.integrate(AppAction::Model(ModelAction::Scenario(
                      ScenarioEdit::NewDispatch)));
      }
      if igButton(const_cstr!("\u{f56c} Add auto dispatch").as_ptr(), v2_0) {
          app.integrate(AppAction::Model(ModelAction::Scenario(
                      ScenarioEdit::NewMovement)));
      }
  }

  igEndChild();
    }

}
