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

use matches::matches;

fn main() {

    mileage::test_lsq_rs();
    return;

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

    // TODO 
    let mut splitsize = 500.0;
    let mut diagram = None;

    // Main loop GUI
    backend_glfw::backend("glrail", |action| {

        // Check for updates in background thread
        doc.receive();

        // Open diagram pane
        if let Some((d_idx,time)) = canvas.active_dispatch {
            if diagram.is_none() {
                if let Some(Some(x)) = doc.get_data().history.get(d_idx) {
                    diagram = Some(diagram::Diagram::from_history(x.clone()));
                }
            }
        } else { diagram = None; }

        // Draw canvas in the whole window
        ui::in_root_window(|| {

            if let Some(ref mut diag) = diagram {

                ui::Splitter::horizontal(&mut splitsize)
                    .left(const_cstr!("canvas").as_ptr(), || { canvas.draw(&mut doc); })
                    .right(const_cstr!("graph").as_ptr(), || { diag.draw(&mut doc, &mut canvas); }); // diagram can show transient rgraphics in canvas

            } else {

                canvas.draw(&mut doc);

            }
        });

        // Continue running.
        !matches!(action, backend_glfw::SystemAction::Close)
    }).unwrap();
}
