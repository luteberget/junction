use backend_glfw::imgui::*;
use const_cstr::*;

pub fn main_menu(show_debug :&mut bool) {
    unsafe {
        if igBeginMenuBar() {

            if igBeginMenu(const_cstr!("File").as_ptr(), true) {
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Edit").as_ptr(), true) {
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("View").as_ptr(), true) {
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Tools").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("View data").as_ptr(), std::ptr::null(), *show_debug, true) {
                    *show_debug = !*show_debug;
                }
                igEndMenu();
            }

            igEndMenuBar();
        }
    }
}

