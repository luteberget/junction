use backend_glfw::imgui::*;
use const_cstr::*;
use crate::ui;

pub fn debug_window(popen :&mut bool) {
    unsafe {
    igBegin(const_cstr!("View data").as_ptr(), popen as _, 0 as _);

    ui::show_text("View data");

    igEnd();
    }
}

