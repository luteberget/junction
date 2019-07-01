// TODO remove this when structs have been taken into use
#![allow(dead_code)]
#![allow(unused_variables)]

mod model;
mod canvas;
mod ui;
mod util;
mod view;
mod objects;
mod viewmodel;

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

    backend_glfw::backend(|_| {
        doc.receive();
        ui::in_root_window(|| {
            canvas.draw(&mut doc);
        });
        true
    }).unwrap();
}
