use imgui_sys_bindgen::sys::*;
use const_cstr::const_cstr;

mod pt;
mod symset;
mod grid_canvas;
mod renderrotated;

fn main() {
    let mut grid_canvas_obj = grid_canvas::SchematicCanvas::new();
    backend_glfw::backend(|_| {
        unsafe { 
            igBegin(const_cstr!("grid").as_ptr(), std::ptr::null_mut(), 0 as _ );
            let size = igGetContentRegionAvail_nonUDT2();
            grid_canvas::schematic_canvas(&ImVec2 { x: size.x, y: size.y }, &mut grid_canvas_obj);
            igEnd(); 
        }
        true
    }).unwrap();
}




