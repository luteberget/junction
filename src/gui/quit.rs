use const_cstr::*;
use crate::document::Document;
use crate::app::Windows;
use crate::gui::widgets;
use crate::file;
use log::*;

pub fn quit_window(doc :&mut Document, show_windows :&mut Windows) -> bool {
    unsafe {
    use backend_glfw::imgui::*;
    let mut quit = false;
    let name = const_cstr!("Save before exit?").as_ptr();
    if !igIsPopupOpen(name) { igOpenPopup(name); }
    if igBeginPopupModal(name, &mut show_windows.quit, 0 as _) {
        widgets::show_text("Save file before closing program?");
        let yes = const_cstr!("Yes").as_ptr();
        let no = const_cstr!("No").as_ptr();
        let cancel = const_cstr!("Cancel").as_ptr();
        if igButton(yes, ImVec2{ x: 80.0, y: 0.0 }) {
            let model = doc.model().clone();
            match file::save_interactive(model) {
                Ok(Some(_)) => { quit = true; },
                Ok(None) => { show_windows.quit = false; },
                Err(e) => { error!("Could not save file {:?}", e); },
            };
        }
        if igButton(no, ImVec2{ x: 80.0, y: 0.0 }) {
            quit = true;
        }
        if igButton(cancel, ImVec2{ x: 80.0, y: 0.0 }) {
            show_windows.quit = false;
        }
        igEndPopup();
    }
    quit
    }
}
