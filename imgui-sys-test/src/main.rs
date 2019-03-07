use std::ffi::CString;
use std::ptr;
use const_cstr::const_cstr;

mod sdlinput;


mod app;



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
    let mut app = app::App::new();

    //let mut action_queue = Vec::new();

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

    //let win1 = CString::new("sidebar1").unwrap();

    unsafe {
        //(*imgui_sys_bindgen::sys::igGetIO()).IniFilename = ptr::null_mut();
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

              use self::app::*;
              use imgui_sys_bindgen::sys::*;
              let v2_0 = ImVec2 { x: 0.0, y: 0.0 };

              unsafe {
                  igShowDemoWindow(ptr::null_mut());

                  igBegin(const_cstr!("Sidebar").as_ptr(), ptr::null_mut(), 0);
                  
                  if igCollapsingHeader(const_cstr!("All objects").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      for (i,e) in app.model.inf.entities.iter().enumerate() {
                          match e {
                              Some(Entity::Track(_))  => { 
                                  let s = CString::new(format!("Track##{}", i)).unwrap();
                                  if igSelectable(s.as_ptr(),
                                                  app.view.selected_object == Some(i), 0, v2_0) {
                                      println!("SET {}", i);
                                      app.view.selected_object = Some(i);
                                  }
                              },
                              Some(Entity::Node(p,_))   => { 
                                  let s = CString::new(format!("Node @ {}##{}", p,i)).unwrap();
                                  if igSelectable(s.as_ptr(), 
                                                  app.view.selected_object == Some(i), 0, v2_0) {
                                      println!("SET NODE {}", i);
                                      app.view.selected_object = Some(i);
                                  }
                              },
                              Some(Entity::Object(_)) => { 
                                  igText(const_cstr!("Object#0").as_ptr()); 
                              },
                              _ => {},
                          }
                      }
                  }

                  if igCollapsingHeader(const_cstr!("Object properties").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      match &app.view.selected_object {
                          Some(_) => {
                              igText(const_cstr!("Some object IS selected.").as_ptr());
                          },
                          None => {
                              igText(const_cstr!("No objects selected.").as_ptr());
                          }
                      }
                  }

                  if igButton(const_cstr!("Add track").as_ptr(), ImVec2 {x:  0.0, y: 0.0 }) {
                      app.integrate(EditorAction::Inf(
                              InfrastructureEdit::NewTrack(0.0,100.0)));
                  }



                  if igCollapsingHeader(const_cstr!("Routes").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      for r in &app.model.routes {

                      }
                  }
                  if igCollapsingHeader(const_cstr!("Scenarios").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      for r in &app.model.scenarios {

                      }
                  }
                  igEnd();

                  igBegin(const_cstr!("Issues").as_ptr(),ptr::null_mut(),0);
                  for error in &app.model.errors {

                  }
                  igEnd();


                  // CANVAS!

                  igBegin(const_cstr!("Canvas").as_ptr(), ptr::null_mut(), 0);
                  let draw_list = igGetWindowDrawList();
                  igText(const_cstr!("Here is the canvas:").as_ptr());

                  let canvas_pos = igGetCursorScreenPos();
                  let mut canvas_size = igGetContentRegionAvail();
                  if canvas_size.x < 10.0 { canvas_size.x = 10.0 }
                  if canvas_size.y < 10.0 { canvas_size.y = 10.0 }
                  ImDrawList_AddRectFilled(draw_list, canvas_pos,
                                           ImVec2 { x: canvas_pos.x + canvas_size.x,
                                                    y: canvas_pos.y + canvas_size.y, },
                                            60 + (60<<8) + (60<<16) + (255<<24), 
                                            0.0, 0);
                  igInvisibleButton(const_cstr!("canvasbtn").as_ptr(), canvas_size);

                  igEnd();
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
