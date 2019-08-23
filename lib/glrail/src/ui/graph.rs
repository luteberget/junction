use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::json::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::view::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
use crate::dgraph::*;
use crate::command_builder::*;
use std::ptr;
use std::ffi::CString;
use const_cstr::const_cstr;

use imgui_sys_bindgen::sys::ImVec2;

pub fn graph(size: ImVec2, app :&mut App) -> bool {
        let canvas_bg = 60 + (60<<8) + (60<<16) + (255<<24);
    let line_col  = 208 + (208<<8) + (175<<16) + (255<<24);
    let tvd_col  = 175 + (255<<8) + (175<<16) + (255<<24);
    let selected_col  = 175 + (175<<8) + (255<<16) + (255<<24);
    let line_hover_col  = 255 + (50<<8) + (50<<16) + (255<<24);
    // TODO make some colors config struct

    unsafe {

    let io = igGetIO();
    let mouse_pos = (*io).MousePos;

  igBeginChild(const_cstr!("Graph").as_ptr(), size, false, 0);
  let capture_canvas_key = igIsWindowFocused(0);

  let draw_list = igGetWindowDrawList();
  igText(const_cstr!("Here is the graph:").as_ptr());

  // we are in the graph mode, so we should have a selected dispatch
  // TODO this is way too complicated
  let historygraph = match app.model.view.selected_scenario {
      SelectedScenario::Dispatch(d) => {
          if let Some(Scenario::Dispatch(Dispatch { history: Derive::Ok(h), .. })) 
              = app.model.scenarios.get_mut(d) { Some(h) } else { None }
      },
      SelectedScenario::Usage(u,Some(d)) => {
          if let Some(Scenario::Usage(_, Derive::Ok(dispatches))) 
              = app.model.scenarios.get_mut(d) { 
                  if let Some(Dispatch { history: Derive::Ok(h), .. }) 
                      = dispatches.get_mut(d) { Some(h) } else { None }
              } else { None }
      },
      _ => None,
  };


  if let Some(hg) = historygraph {
      let graph = hg.graph(&app.model.inf, &app.model.dgraph.get().unwrap(), &app.model.schematic.get().unwrap());

      // slider time
      let mut time = graph.instant.time as f32;
      let format = const_cstr!("%.3f").as_ptr();
      if igSliderFloat(const_cstr!("Time").as_ptr(), &mut time as *mut _, 0.0, graph.time_range as f32, format, 1.0) {

          // TODO reify to a ScenarioEdit object?
          hg.set_time(time, &app.model.inf, &app.model.dgraph.get().unwrap(), &app.model.schematic.get().unwrap());
      }
      show_text("GRAPH");
  }


  igEndChild();

  capture_canvas_key
    }
}
