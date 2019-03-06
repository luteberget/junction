use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("lib/cimgui/cimgui.cpp")
        .file("lib/cimgui/imgui/imgui.cpp")
        .file("lib/cimgui/imgui/imgui_demo.cpp")
        .file("lib/cimgui/imgui/imgui_draw.cpp")
        .file("lib/cimgui/imgui/imgui_widgets.cpp")
        .compile("libcimgui.a");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Could not create bindings to library");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

}
