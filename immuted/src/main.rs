// TODO remove this when structs have been taken into use
#![allow(dead_code)]
#![allow(unused_variables)]

use const_cstr::*;

mod model;
mod canvas;
mod ui;
mod logview;
mod util;
mod view;
mod objects;
mod viewmodel;
mod dgraph;
mod mileage;
mod interlocking;
mod topology;
mod history;
mod diagram;
mod dispatch;
mod vehicles;

mod config;
mod mainmenu;
mod debug;
mod file;

mod import;

use matches::matches;
use log::*;

pub struct AllState<'a> {
    viewmodel :&'a viewmodel::ViewModel,
    canvas :&'a canvas::Canvas,
    diagram :&'a diagram::Diagram,
    config :&'a config::Config,
}


fn main() {
    use crate::model::*;

    // init logging
    let logstring = logview::StringLogger::init(log::LevelFilter::Trace).unwrap();
    info!("Starting application");

    // Stores lines(tracks), node data, objects, vehicles and dispatches
    // in persistent datastructures, in an undo/redo stack.
    let m : Undoable<Model> = Undoable::new();
    let thread_pool = threadpool::ThreadPool::new(2);

    let mut config = config::Config::default();

    // Embed the model into a viewmodel that calculates derived data
    // in the background.
    let mut doc = viewmodel::ViewModel::new(m, file::FileInfo::empty(), thread_pool.clone());

    // Stores view, selection, and input mode.
    // Edits doc (and calls undo/redo).
    let mut canvas = canvas::Canvas::new();
    let mut diagram = diagram::Diagram::new();

    // TODO 
    let mut splitsize = 500.0;
    let mut show_windows = mainmenu::ShowWindows {
        debug: false,
        config: false,
        log: false,
        vehicles :false,
        quit: false,
        import: import::ImportWindow::new(thread_pool.clone()),
    };

    // Main loop GUI
    backend_glfw::backend(&doc.fileinfo.window_title(), 
                          config.get_font_filename().as_ref().map(|x| x.as_str()), 
                          config.get_font_size(),
                          |action| {
        if matches!(action, backend_glfw::SystemAction::Close) {
            show_windows.quit = true;
        }

        // Check for updates in background thread
        doc.receive(&mut canvas.instant_cache); // TODO avoid explicit cache clearing
        show_windows.import.update();

        // forward time if playing
        if let Some((_,time,play)) = &mut canvas.active_dispatch {
            if *play {
                let dt = unsafe { (*backend_glfw::imgui::igGetIO()).DeltaTime };
                *time += dt*25.0;
            }
        }


        ui::in_root_window(|| {

            mainmenu::main_menu(&mut show_windows, &mut doc, &mut canvas, &mut diagram, &thread_pool);

            if canvas.active_dispatch.is_some() {
                ui::Splitter::vertical(&mut splitsize)
                    .left(const_cstr!("canvas").as_ptr(), || { 
                        canvas.draw(&mut doc, &config); })
                    .right(const_cstr!("graph").as_ptr(), || { 
                        diagram.draw(&mut doc, &mut canvas, &config); });

            } else {
                canvas.draw(&mut doc, &config);
            }
        });

        if show_windows.debug {
            let state = AllState {
                viewmodel: &doc,
                canvas: &canvas,
                diagram: &diagram,
                config: &config,
            };
            debug::debug_window(&mut show_windows.debug, state);
        }

        if show_windows.config {
            config::edit_config_window(&mut show_windows.config, &mut config);

        }
        if show_windows.log {
            logview::view_log(&mut show_windows.log, &logstring);
        }

        if show_windows.vehicles {
            vehicles::edit_vehicles_window(&mut show_windows.vehicles, &mut doc);
        }

        if show_windows.import.open {
            show_windows.import.draw(&mut doc);
        }


        let really_quit = if show_windows.quit { 
            if doc.fileinfo.unsaved {
                quit_window(&mut doc, &mut show_windows) 
            } else { true }
        } else { false };

        // Continue running.
        !really_quit
    }).unwrap();
}

pub fn quit_window(doc :&mut viewmodel::ViewModel, show_windows :&mut mainmenu::ShowWindows) -> bool {
    unsafe {
    use backend_glfw::imgui::*;
    let mut quit = false;
    let name = const_cstr!("Save before exit?").as_ptr();
    if !igIsPopupOpen(name) { igOpenPopup(name); }
    if igBeginPopupModal(name, &mut show_windows.quit, 0 as _) {
        ui::show_text("Save file before closing program?");
        let yes = const_cstr!("Yes").as_ptr();
        let no = const_cstr!("No").as_ptr();
        let cancel = const_cstr!("Cancel").as_ptr();
        if igButton(yes, ImVec2{ x: 80.0, y: 0.0 }) {
            let model = doc.get_undoable().get().clone();
            match file::save_interactive(&mut doc.fileinfo, model) {
                Ok(true) => { quit = true; },
                Ok(false) => { show_windows.quit = false; },
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
