pub mod app;
pub mod config;
pub mod document;
pub mod file;
pub mod gui;
pub mod util;

use log::*;
use crate::app::*;

fn main() {
    // Init logging
    let logstring = gui::logview::StringLogger::init(log::LevelFilter::Trace).unwrap();
    info!("Starting {} v{}.", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // User config not directly related to model or ui state. (colors, fonts, etc.)
    let config = config::Config::load();
    let background_jobs = app::BackgroundJobs::new();

    // Create an empty, untitled document
    // TODO: command line read from file
    let document = document::Document::empty(background_jobs.clone());

    // Additional windows are closed.
    let windows = app::Windows::closed();

    let mut app = app::App {
        document: document,
        log: logstring,
        config :config,
        windows: windows,
        background_jobs: background_jobs,
    };


    backend_glfw::backend(&app.document.fileinfo.window_title(),
                          app.config.get_font_filename().as_ref().map(|x| x.as_str()),
                          app.config.get_font_size(),
                          |action| {
                              
        match action {
            // Window system requested quit (clicked ALT+F4, close button, or similar)
            backend_glfw::SystemAction::Close => { app.windows.quit = true; },
            _ => {},
        };

        // Check background threads for updates
        app.document.check();
        // TODO app.windows.import.check();
        // TODO app.windows.synthesis.check()

        // Advance time in animations
		let dt = unsafe { (*backend_glfw::imgui::igGetIO()).DeltaTime } as f64;
        if let Some(d) = &mut app.document.dispatch { d.advance(dt); }

        // Draw and interact with GUI
        let continue_running = gui::main(&mut app);
        return continue_running;
    }).unwrap();
}
