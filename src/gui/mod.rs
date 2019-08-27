pub mod logview;
pub mod widgets;
pub mod mainmenu;
pub mod debug;
pub mod vehicles;

pub use backend_glfw::imgui::ImVec2;

use crate::app::*;

pub fn main(app :&mut App) -> bool {

    // global hotkeys

    // Main window
    widgets::in_root_window(|| {

        // top menu bar
        mainmenu::main_menu(app);

    });


    // Other windows
    logview::view_log(&mut app.windows.log, &app.log);
    app.windows.debug = debug::debug_window(app.windows.debug, &app);
    vehicles::edit_vehicles_window(&mut app.windows.vehicles, &mut app.document);

    let quit = false;
    !quit
}
