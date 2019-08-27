pub mod logview;
pub mod widgets;
pub mod mainmenu;
pub mod debug;
pub mod vehicles;
pub mod config;
pub mod quit;
pub mod infview;
pub mod plan;
pub mod diagram;
pub mod dispatch;
pub mod menus;
pub mod draw_inf;
pub mod keys;

pub use backend_glfw::imgui::ImVec2;

use crate::app::*;
use crate::document::*;

use const_cstr::*;

pub fn main(app :&mut App) -> bool {

    // keyboard commands (ctrl+s for save, etc. + a/s/d for tool selection)
    keys::keys(app);

    // Main window
    widgets::in_root_window(|| {

        // top menu bar
        mainmenu::main_menu(app);

        // Three main window arrangements:
        // 1. Infrastructure only (diagram_view = None)
        // 2. Manual dispatch view (diagram_view = Some(DiagramView::Manual(...)))
        // 3. Auto-dispatch view (diagram_view = Some(DiagramView::Manual(...)))
        match &app.document.dispatch_view {
            None => { infview::inf_view(app); },
            Some(_) => {

                // TODO splitting size logic here?
                if app.windows.diagram_split.is_none() { app.windows.diagram_split = Some(500.0); } 

                widgets::Splitter::vertical(app.windows.diagram_split.as_mut().unwrap())
                    .left(const_cstr!("inf_canv").as_ptr(), || {
                        infview::inf_view(app); })
                    .right(const_cstr!("dia_dptch").as_ptr(), || {
                        dispatch::dispatch_view(app); });
            },
        }
    });

    // Other windows
    logview::view_log(&mut app.windows.log, &app.log);
    app.windows.debug = debug::debug_window(app.windows.debug, &app);
    vehicles::edit_vehicles_window(&mut app.windows.vehicles, &mut app.document);
    config::edit_config_window(&mut app.windows.config, &mut app.config);

    // Quit dialog
    let really_quit = if app.windows.quit {
		if app.document.fileinfo.unsaved {
			quit::quit_window(&mut app.document, &mut app.windows)
		} else { true }
	} else { false };

    !really_quit
}
