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

use matches::matches;
use log::*;

pub struct AllState<'a> {
    fileinfo :&'a FileInfo,
    viewmodel :&'a viewmodel::ViewModel,
    canvas :&'a canvas::Canvas,
    diagram :&'a diagram::Diagram,
    config :&'a config::Config,
}

#[derive(Debug)]
pub struct FileInfo {
    pub filename :Option<String>,
    pub unsaved :bool,
}

impl FileInfo {
    pub fn empty() -> Self {
        FileInfo {
            filename :None,
            unsaved :false,
        }
    }

    pub fn set_unsaved(&mut self, ctx :&mut backend_glfw::Ctx) {
        if !self.unsaved {
            self.unsaved = true;
            ctx.set_window_title(&self.window_title());
        }
    }

    pub fn window_title(&self) -> String {
        format!("{}{} - Junction", if self.unsaved {"*"}  else { "" },
                                   self.filename.as_ref().map(|x| x.as_str()).unwrap_or("Untitled"))
    }
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
    let mut fileinfo = FileInfo::empty();

    // Embed the model into a viewmodel that calculates derived data
    // in the background.
    let mut doc = viewmodel::ViewModel::new(m, thread_pool.clone());

    // Stores view, selection, and input mode.
    // Edits doc (and calls undo/redo).
    let mut canvas = canvas::Canvas::new();
    let mut diagram = diagram::Diagram::new();

    // TODO 
    let mut splitsize = 500.0;
    let mut show_debug = false;
    let mut show_config = false;
    let mut show_log = false;
    let mut show_vehicles = false;

    // Main loop GUI
    backend_glfw::backend("glrail", config.get_font_filename().as_ref().map(|x| x.as_str()), |ctx, action| {
        fileinfo.set_unsaved(ctx);

        // Check for updates in background thread
        doc.receive(&mut canvas.instant_cache); // TODO avoid explicit cache clearing

        // forward time if playing
        if let Some((_,time,play)) = &mut canvas.active_dispatch {
            if *play {
                let dt = unsafe { (*backend_glfw::imgui::igGetIO()).DeltaTime };
                *time += dt*25.0;
            }
        }


        ui::in_root_window(|| {
            mainmenu::main_menu(&mut show_config, &mut show_debug, &mut show_log, &mut show_vehicles);
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

        if show_debug {
            let state = AllState {
                fileinfo: &fileinfo,
                viewmodel: &doc,
                canvas: &canvas,
                diagram: &diagram,
                config: &config,
            };
            debug::debug_window(&mut show_debug, state);
        }

        if show_config {
            config::edit_config_window(&mut show_config, &mut config);

        }
        if show_log {
            logview::view_log(&mut show_log, &logstring);
        }

        if show_vehicles {
            vehicles::edit_vehicles_window(&mut show_vehicles, &mut doc);
        }

        // Continue running.
        !matches!(action, backend_glfw::SystemAction::Close)
    }).unwrap();
}
