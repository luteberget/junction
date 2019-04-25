use const_cstr::const_cstr;

use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::text::*;
use junc_app::AppEvent;

pub fn menubar() -> Option<AppEvent> {
    let mut output = None;
    unsafe {
        if igBeginMainMenuBar() {
            if igBeginMenu(const_cstr!("File").as_ptr(), true)
            {
                if igMenuItemBool(const_cstr!("Quit").as_ptr(), 
                                  const_cstr!("CTRL+Q").as_ptr(), false, true) {
                    output = Some(AppEvent::Quit);
                }
                //ShowExampleMenuFile();
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Edit").as_ptr(), true) {
                //if iGMenuItem("Undo", "CTRL+Z")) {}
                //if (ImGui::MenuItem("Redo", "CTRL+Y", false, false)) {}  // Disabled item
                //ImGui::Separator();
                //if (ImGui::MenuItem("Cut", "CTRL+X")) {}
                //if (ImGui::MenuItem("Copy", "CTRL+C")) {}
                //if (ImGui::MenuItem("Paste", "CTRL+V")) {}
                igEndMenu();
            }

            igSameLine(0.0, -1.0);
            show_text("Junction [unnamed file] *");

            igEndMainMenuBar();
        }

    }

    output
}
