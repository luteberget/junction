// TODO remove this when structs have been taken into use
#![allow(dead_code)]
#![allow(unused_variables)]

mod model;
mod canvas;
mod ui;
mod util;
mod view;
mod objects;

#[derive(Default)]
pub struct Derived {
    epoch :usize,
    nodes :Option<Nodes>,
    dgraph :Option<DGraph>,
    interlocking :Option<Interlocking>,
    history :Option<Vec<History>>,
}

pub struct Nodes {}
pub struct DGraph {}
pub struct Interlocking {}
pub struct History {}

fn main() {
    use crate::model::*;

    // Stores lines(tracks), node data, objects, vehicles and dispatches
    // in persistent datastructures, in an undo/redo stack.
    let mut doc : Undoable<Model> = Undoable::new();

    // Stores view, selection, and input mode.
    // Edits doc (and calls undo/redo).
    let mut canvas = canvas::Canvas::new();

    // Stores railway nodes (derived from lines), 
    // background thread computation results s.a.:
    //  - dgraph, - interlocking, - dispatch histories.
    let mut derived :Derived = Default::default();

    backend_glfw::backend(|_| {
        ui::in_root_window(|| {
            canvas.draw(&mut doc, &mut derived);
        });
        true
    }).unwrap();
}
