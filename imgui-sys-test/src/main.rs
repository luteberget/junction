mod sdlinput;

fn gui_init() {
    use imgui_sys_bindgen::sys::*;
    use std::ptr;
    unsafe {
        let _ig = igCreateContext(ptr::null_mut());
        let _io = igGetIO();
        igStyleColorsDark(ptr::null_mut());
    }
}

//fn gui_frame() {
//        let io = igGetIO();
//        igNewFrame();
//        //igRender();
//}

fn gui_destroy() {
}


fn main() -> Result<(), String>{
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("glrail", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .map_err(|e| format!("{}", e))?;

    let _gl_context = window.gl_create_context().expect("Couldn't create GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);


    let mut canvas = window.into_canvas()
        .target_texture()
        //.present_vsync()
        .build()
        .map_err(|e| format!("{}", e))?;

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator : sdl2::render::TextureCreator<_> 
        = canvas.texture_creator();


    gui_init();

    let mut imgui_renderer = imgui_sys_opengl::Renderer::new(|s| video_subsystem.gl_get_proc_address(s) as _);
    let mut imgui_sdl = sdlinput::ImguiSdl2::new();


    use sdl2::event::Event;
    fn not_mousemotion(ev :&Event) -> bool {
        if let &Event::MouseMotion { .. } = ev { false } else { true }
    }
    fn exit_on(ev :&Event) -> bool {
        if let &Event::Quit { .. } = ev { true } else { false }
    }


    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i :i64 = 0;
    let mut events = |mut f: Box<FnMut(sdl2::event::Event) -> bool>| {
        'running: loop {
            let mut render = false;
              let event =  event_pump.wait_event();
              imgui_sdl.handle_event(&event);
              if exit_on(&event) { break 'running; }
              if not_mousemotion(&event) { render = true; }

              for event2 in event_pump.poll_iter() {
                  imgui_sdl.handle_event(&event2);
                  if exit_on(&event) { break 'running; }
                  if not_mousemotion(&event) { render = true; }
              }

            for _ in  (1..=3) {
              for event2 in event_pump.poll_iter() {
                  imgui_sdl.handle_event(&event2);
                  if exit_on(&event) { break 'running; }
              }

              let c = sdl2::pixels::Color::RGB(15,15,15);
              //println!("frame! color {:?}", c);
              canvas.set_draw_color(c);
              canvas.clear();
              //gui_frame();

              imgui_sdl.frame(&canvas.window(), &event_pump.mouse_state());

              unsafe {
                  use imgui_sys_bindgen::sys::*;
                  use std::ptr;
                  igShowDemoWindow(ptr::null_mut());
              }

              imgui_renderer.render();
              canvas.present();
            }
        }
    };

    events(Box::new(|ev| {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        match ev {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return true;
                },

                _ => {},

        }

        return false;

    }));

    gui_destroy();

    Ok(())
}
