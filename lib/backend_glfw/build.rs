use std::env;
use std::path::PathBuf;


fn main() {

    cc::Build::new()
        .cpp(true)
        .file("lib/imgui_impl_glfw.cpp")
        .file("lib/imgui_impl_opengl3.cpp")
        .file("lib/main.cpp")
        .file("../imgui-sys-bindgen/lib/cimgui.cpp")
        .file("../imgui-sys-bindgen/lib/imgui.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_demo.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_draw.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_widgets.cpp")
        .file("lib/gl3w/GL/gl3w.c")
        .include("../imgui-sys-bindgen/lib")
        .include("lib/gl3w")
        .include("lib/glfw/include")
        .compile("libimgui_glfw_opengl3.a");

    // add glfw to linker args
    //

    println!("cargo:rustc-link-lib=glfw3");
    //println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=opengl32");


    let bindings = bindgen::Builder::default()
        .header("../imgui-sys-bindgen/wrapper.h")
        .generate()
        .expect("Could not create bindings to library");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");


}
