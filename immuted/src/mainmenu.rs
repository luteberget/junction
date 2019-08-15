use backend_glfw::imgui::*;
use const_cstr::*;

pub fn main_menu(show_config :&mut bool, show_debug :&mut bool, show_log :&mut bool) {
    unsafe {
        if igBeginMenuBar() {

            if igBeginMenu(const_cstr!("File").as_ptr(), true) {
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Edit").as_ptr(), true) {
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("View").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("Log window").as_ptr(), std::ptr::null(), *show_log, true) {
                    *show_log = !*show_log;
                }
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Tools").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("View data").as_ptr(), std::ptr::null(), *show_debug, true) {
                    *show_debug = !*show_debug;
                }
                if igMenuItemBool(const_cstr!("Configure colors").as_ptr(), std::ptr::null(), *show_config, true) {
                    *show_config = !*show_config;
                }
                igEndMenu();
            }

            igEndMenuBar();
        }
    }
}

