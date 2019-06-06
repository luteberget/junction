
fn main() {
    cc::Build::new()
        .cpp(true)
        .file("lib/imgui_impl_win32.cpp")
        .file("lib/imgui_impl_dx9.cpp")
        .file("lib/main.cpp")
        .file("../imgui-sys-bindgen/lib/imgui.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_demo.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_draw.cpp")
        .file("../imgui-sys-bindgen/lib/imgui_widgets.cpp")
        .include("../imgui-sys-bindgen/lib")
        .compile("libimgui_win32_directx9.a");
}
