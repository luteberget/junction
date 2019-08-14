// TODO remove this when structs have been taken into use
#![allow(dead_code)]
#![allow(unused_variables)]

use const_cstr::*;

mod model;
mod canvas;
mod ui;
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

mod colors;

use matches::matches;

fn main() {
    use crate::model::*;

    // Stores lines(tracks), node data, objects, vehicles and dispatches
    // in persistent datastructures, in an undo/redo stack.
    let m : Undoable<Model> = Undoable::new();
    let thread_pool = threadpool::ThreadPool::new(2);

    // Embed the model into a viewmodel that calculates derived data
    // in the background.
    let mut doc = viewmodel::ViewModel::new(m, thread_pool.clone());

    // Stores view, selection, and input mode.
    // Edits doc (and calls undo/redo).
    let mut canvas = canvas::Canvas::new();
    let mut diagram = diagram::Diagram::new();

    // TODO 
    let mut splitsize = 500.0;

    // Main loop GUI
    backend_glfw::backend("glrail", |action| {

        // Check for updates in background thread
        doc.receive(&mut canvas.instant_cache); // TODO avoid explicit cache clearing

        // forward time if playing
        if let Some((_,time,play)) = &mut canvas.active_dispatch {
            if *play {
                let dt = unsafe { (*backend_glfw::imgui::igGetIO()).DeltaTime };
                *time += dt*25.0;
            }
        }

        // Draw canvas in the whole window
        ui::in_root_window(|| {

            if canvas.active_dispatch.is_some() {
                ui::Splitter::vertical(&mut splitsize)
                    .left(const_cstr!("canvas").as_ptr(), || { 
                        canvas.draw(&mut doc); })
                    .right(const_cstr!("graph").as_ptr(), || { 
                        diagram.draw(&mut doc, &mut canvas); });

            } else {
                canvas.draw(&mut doc);
            }
        });

        // Continue running.
        !matches!(action, backend_glfw::SystemAction::Close)
    }).unwrap();
}
