// TODO remove this when structs have been taken into use
#![allow(dead_code)]
#![allow(unused_variables)]

mod model;
mod canvas;
mod ui;
mod util;
mod view;
mod objects;

fn main() {
    use crate::model::*;
    use backend_glfw::imgui::*;
    let mut doc : Undoable<Model> = Undoable::new();
    let mut canvas = canvas::Canvas::new();
    backend_glfw::backend(|_| {
        ui::in_root_window(|| {
            let size = unsafe { igGetContentRegionAvail_nonUDT2().into() };
            canvas.draw(&mut doc,size);
        });
        true
    }).unwrap();
}
