


use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::json::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
use crate::command_builder::*;
use std::ptr;
use std::ffi::CString;
use const_cstr::const_cstr;

pub fn command(loc :ImVec2, app :&mut App) -> bool {
    let v2_0 = ImVec2 { x: 0.0, y: 0.0 };
    unsafe {

        let mut capture_command_key = false;

  let mut overlay_start = || {
      igSetNextWindowBgAlpha(0.75);
      igSetNextWindowPos(loc,
      //igSetNextWindowPos((*viewport).Pos, 
         ImGuiCond__ImGuiCond_Always as _, v2_0);
      igPushStyleColor(ImGuiCol__ImGuiCol_TitleBgActive as _, 
                     ImVec4 { x: 1.0, y: 0.65, z: 0.7, w: 1.0 });
      igBegin(const_cstr!("Command").as_ptr(), ptr::null_mut(),
        (ImGuiWindowFlags__ImGuiWindowFlags_AlwaysAutoResize | 
        ImGuiWindowFlags__ImGuiWindowFlags_NoMove | 
        ImGuiWindowFlags__ImGuiWindowFlags_NoResize) as _
        );

      capture_command_key = igIsWindowFocused(0);
  };
      
  let overlay_end = || {
      igEnd();
      igPopStyleColor(1);
  };
  
  // Overlay command builder
  let mut new_screen_func = None;
  let mut alb_execute = false;
  let mut alb_cancel = false;
  if let Some(ref mut command_builder) = &mut app.command_builder {
      match command_builder.current_screen() {
          CommandScreen::Menu(Menu { choices }) => {
              // Draw menu
              //
              overlay_start();

              for (i,c) in choices.iter().enumerate() {
                igPushIDInt(i as _);
                  if igSelectable(const_cstr!("##mnuitm").as_ptr(), false, 0, v2_0) {
                      new_screen_func = Some(c.2);
                  }

                  igSameLine(0.0, -1.0);

                  let s = CString::new(format!("{} - ", c.0)).unwrap();
                  igTextColored( ImVec4 { x: 0.95, y: 0.5, z: 0.55, w: 1.0 }, s.as_ptr());

                  igSameLine(0.0, -1.0);
                  //igText(const_cstr!("context").as_ptr());
                  show_text(&c.1);
                igPopID();
              }

              overlay_end();

          },
          CommandScreen::ArgumentList(alb) => {
              overlay_start();
              for (i,(name, status, arg)) in alb.arguments.iter_mut().enumerate() {
                  igPushIDInt(i as _);

                  let s = CString::new(name.as_str()).unwrap();
                  match status {
                      ArgStatus::Done => {
                          let c = ImVec4 { x: 0.55, y: 0.55, z: 0.80, w: 1.0 };
                          igTextColored(c, s.as_ptr());
                          igSameLine(0.0,-1.0);
                          match arg {
                              Arg::Id(Some(x)) => {
                                  show_text(&format!("obj:{}", x));
                              },
                              Arg::Float(val) => {
                                  show_text(&format!("{}", val));
                              },
                              _ => { panic!(); },
                          }
                      },
                      ArgStatus::NotDone => {
                          let c = ImVec4 { x: 0.95, y: 0.5,  z: 0.55, w: 1.0 };
                          igTextColored(c, s.as_ptr());
                          igSameLine(0.0,-1.0);
                          match arg {
                              Arg::Id(x) => {
                                  show_text(&format!("obj:{:?}", x));
                              },
                              Arg::Float(ref mut val) => {
                                igInputFloat(const_cstr!("##num").as_ptr(), 
                                             val as *mut _, 0.0, 1.0, 
                                             const_cstr!("%g").as_ptr(), 0);
                              },
                          }
                      },
                  };

                  igPopID();
              }

              if igButton(const_cstr!("\u{f04b} Execute").as_ptr(), v2_0) {
                  alb_execute = true;
              }

              igSameLine(0.0,-1.0);
              if igButton(const_cstr!("\u{f05e} Cancel").as_ptr(), v2_0) {
                  alb_cancel = true;
              }
              overlay_end();
          },
          _ => {},
      }
  }

  if let Some(f) = new_screen_func {
      if let Some(s) = f(app) {
          if let Some(ref mut c) = app.command_builder {
              c.push_screen(s);
          }
      } else {
          app.command_builder = None;
      }
  }

  if alb_execute {
      use std::mem;
      let cb = mem::replace(&mut app.command_builder, None);
      if let Some(cb) = cb {
          cb.execute(app);
      }
  }

  if alb_cancel {
      app.command_builder = None;
  }
    capture_command_key
    }

}
