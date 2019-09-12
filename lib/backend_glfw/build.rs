use std::env;
use std::path::PathBuf;


fn main() {

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++11")
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
    //

    #[cfg(target_os = "windows")]
    {
    println!("cargo:rustc-link-lib=glfw3");
    //println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=opengl32");
    }


    #[cfg(target_os = "linux")]
    {
    println!("cargo:rustc-link-lib=glfw");
    println!("cargo:rustc-link-lib=GL");
    }


    #[cfg(target_os = "osx")]
    {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={}/bin", manifest_dir);
    println!("cargo:rustc-link-lib=glfw3");
    println!("cargo:rustc-link-lib=GL");
    }





    let bindings = bindgen::Builder::default()
        .header("../imgui-sys-bindgen/wrapper.h")
        .generate()
        .expect("Could not create bindings to library");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");


}
