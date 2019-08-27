pub mod logview;
pub mod widgets;
pub mod mainmenu;

pub use backend_glfw::imgui::ImVec2;

use crate::app::*;

pub fn main(app :&mut App) -> bool {

    // global hotkeys

    // top menu bar
    mainmenu::main_menu(app);

    let quit = false;
    !quit
}
