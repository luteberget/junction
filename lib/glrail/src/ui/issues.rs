use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
//use crate::issues::*;
use std::ptr;
use std::ffi::CString;
use const_cstr::const_cstr;

pub fn issues(size :ImVec2, app :&mut App) -> bool {
    unsafe {
  igBeginChild(const_cstr!("Issues").as_ptr(),size, false,0);
  igText(const_cstr!("Here are the issues:").as_ptr());
  for issue in app.model.iter_issues() {

  }
  igEndChild();
    }
    false
}
