
fn main() {

    cc::Build::new()
        .cpp(true)
        .file("lib/imgui_impl_glfw.cpp")
        .file("lib/imgui_impl_opengl3.cpp")
        .file("lib/main.cpp")
        .file("../imgui-sys-bindgen/lib/imgui.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_demo.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_draw.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_widgets.cpp")
        .file("lib/gl3w/GL/gl3w.c")
        .include("../imgui-sys-bindgen/lib")
        .include("lib/gl3w")
        .compile("libimgui_glfw_opengl3.a");

    // add glfw to linker args
    //

    println!("cargo:rustc-link-lib=glfw");
    println!("cargo:rustc-link-lib=GL");

}
