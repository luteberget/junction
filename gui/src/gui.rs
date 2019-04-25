use crate::sdlinput;

pub fn main_loop(mut f :impl FnMut(&mut Iterator<Item = sdl2::event::Event>) -> bool ) -> Result<(), String> {
    // GUI main loop
    //

    let sdl_context = sdl2::init()?;
    let event_subsystem = sdl_context.event()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Junction [unnamed file]", 800, 600)
        .opengl()
        .resizable()
        .position_centered()
        .build()
        .map_err(|e| format!("{}", e))?;

    let _gl_context = window.gl_create_context()
        .expect("Couldn't create GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

    let mut canvas = window.into_canvas()
        .target_texture()
        //.present_vsync()
        .build()
        .map_err(|e| format!("{}", e))?;

    imgui_init();
    let mut imgui_renderer = imgui_sys_opengl::Renderer::new(
        |s| video_subsystem.gl_get_proc_address(s) as _);
    let mut imgui_sdl = sdlinput::ImguiSdl2::new();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut events = Vec::new();
    'running: loop {
        events.push(event_pump.wait_event());

        // Every time we have waited, draw 3 frames to catch
        // single-frame delays inherent in the IMGUI workings.
        for _ in 1..=3 {
            for ev in event_pump.poll_iter() {
                events.push(ev);
            }

            let c = sdl2::pixels::Color::RGB(200,200,200);
            canvas.set_draw_color(c);
            canvas.clear();
            imgui_sdl.frame(&canvas.window(), &event_pump.mouse_state());
            for ev in &events {
                imgui_sdl.handle_event(ev);
            }
            let cont = f(&mut events.drain(0..));
            imgui_renderer.render();
            canvas.present();
            if !cont { break 'running; }
        }
    }

    Ok(())
}


fn imgui_init() {
    use imgui_sys_bindgen::sys::*;
    use std::ptr;
    unsafe {
        let _ig = igCreateContext(ptr::null_mut());
        let _io = igGetIO();
        igStyleColorsDark(ptr::null_mut());


        //        igStyleColorsLight(ptr::null_mut());
        //ImFontAtlas_AddFontFromFileTTF((*io).Fonts,
        ////       //const_cstr!("DejaVuSansMono.ttf").as_ptr(),
        //       const_cstr!("Roboto-Medium.ttf").as_ptr(),
        //       16.0, ptr::null(), ptr::null());
        ////ImFontAtlas_AddFontDefault((*io).Fonts, ptr::null());

        //let config = ImFontConfig_ImFontConfig();
        //(*config).MergeMode = true;
        //(*config).GlyphMinAdvanceX = 16.0;
        //let ranges : [std::os::raw::c_ushort;3] = [0xf000, 0xf82f, 0x0];
        ////#define ICON_MIN_FA 0xf000
        ////#define ICON_MAX_FA 0xf82f

        //ImFontAtlas_AddFontFromFileTTF((*io).Fonts,
        //    const_cstr!("fa-solid-900.ttf").as_ptr(),
        //    14.0,  config, &ranges as _ );

        //ImFontAtlas_Build((*io).Fonts);

        (*igGetIO()).ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard as i32;

    }

}

