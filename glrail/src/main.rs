use std::ffi::{CString, CStr};
use std::ptr;
use const_cstr::const_cstr;
use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::text::*;
use imgui_sys_bindgen::json::*;

mod sdlinput;

// App
mod app;
mod background;
mod command_builder;
mod ui;
mod wake;
pub use crate::wake::wake;

// Domain
mod model;
mod infrastructure;
mod dgraph;
mod schematic;
mod view;
mod vehicle;
mod selection;
mod interlocking;
mod scenario;
mod issue;
mod analysis;
mod graph;


use self::app::*;
use self::model::*;
use self::view::*;
use self::infrastructure::*;
use self::command_builder::*;
use self::selection::*;
use self::scenario::*;
use crate::dgraph::*;


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


/*
fn linux_backend_begin(handle_event :impl FnMut(Event), draw :impl FnMut()) -> Result<(), String> {

    let sdl_context = sdl2::init()?;
    let event_subsystem = sdl_context.event()?;
    let video_subsystem = sdl_context.video()?;

    video_subsystem.gl_attr().set_context_profile(sdl2::video::GLProfile::Core);
    video_subsystem.gl_attr().set_context_version(3,0);
    //video_subsystem.gl_attr().set_context_minor_version(2);
    

    let window = video_subsystem
        .window("glrail", 800, 600)
        .opengl()
        .resizable()
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

        
    let texture_creator : sdl2::render::TextureCreator<_> 
        = canvas.texture_creator();

    gui_init();
    let io = unsafe { imgui_sys_bindgen::sys::igGetIO() };

    unsafe {
            use imgui_sys_bindgen::sys::*;
        igStyleColorsLight(ptr::null_mut());
        ImFontAtlas_AddFontFromFileTTF((*io).Fonts, 
               const_cstr!("Roboto-Medium.ttf").as_ptr(),
               16.0, ptr::null(), ptr::null());

        let config = ImFontConfig_ImFontConfig();
        (*config).MergeMode = true;
        (*config).GlyphMinAdvanceX = 16.0;
        let ranges : [std::os::raw::c_ushort;3] = [0xf000, 0xf82f, 0x0];
        //#define ICON_MIN_FA 0xf000
        //#define ICON_MAX_FA 0xf82f

        ImFontAtlas_AddFontFromFileTTF((*io).Fonts,
            const_cstr!("fa-solid-900.ttf").as_ptr(),
            14.0,  config, &ranges as _ );

        ImFontAtlas_Build((*io).Fonts);
    }

        let mut imgui_renderer = imgui_sys_opengl::Renderer::new(|s| video_subsystem.gl_get_proc_address(s) as _);
    let mut imgui_sdl = sdlinput::ImguiSdl2::new();

    use sdl2::event::Event;

    
        unsafe {
        use imgui_sys_bindgen::sys::*;
        //(*imgui_sys_bindgen::sys::igGetIO()).IniFilename = ptr::null_mut();
        (*igGetIO()).ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard as i32;

        //igMayaStyle();
        //CherryTheme();
    }

    'running: loop {
            let mut render = false;
              let event =  event_pump.wait_event();
              imgui_sdl.handle_event(&event);
              if exit_on(&event) { break 'running; }
              if not_mousemotion(&event) { render = true; }
              app_event(&event, &mut app, capture_command_key, capture_canvas_key);

              for event2 in event_pump.poll_iter() {
                  imgui_sdl.handle_event(&event2);
                  if exit_on(&event2) { break 'running; }
                  if not_mousemotion(&event2) { render = true; }
                  app_event(&event2, &mut app, capture_command_key, capture_canvas_key);
              }

            for _ in 1..=3 {
              for event2 in event_pump.poll_iter() {
                  imgui_sdl.handle_event(&event2);
                  if exit_on(&event2) { break 'running; }
                  app_event(&event2, &mut app, capture_command_key, capture_canvas_key);
              }

              let c = sdl2::pixels::Color::RGB(15,200,150);
              //println!("frame! color {:?}", c);
              canvas.set_draw_color(c);
              canvas.clear();
              //gui_frame();

              imgui_sdl.frame(&canvas.window(), &event_pump.mouse_state());

              imgui_renderer.render();
              canvas.present();


              if app.want_to_quit {
                  break 'running;
              }
            }
        }


    gui_destroy();

    Ok(())
}
*/


fn main() -> Result<(), String>{
    use sdl2::event::Event;

    use log::LevelFilter;
    simple_logging::log_to_stderr(LevelFilter::Warn);

    let json_types: [*const i8; 6] = [
        const_cstr!("Null").as_ptr(),
        const_cstr!("Bool").as_ptr(),
        const_cstr!("Num").as_ptr(),
        const_cstr!("Text").as_ptr(),
        const_cstr!("Obj").as_ptr(),
        const_cstr!("Arr").as_ptr(),
    ];


    let mut app = app::App::new();


    fn not_mousemotion(ev :&Event) -> bool {
        if let &Event::MouseMotion { .. } = ev { false } else { true }
    }
    fn exit_on(ev :&Event) -> bool {
        if let &Event::Quit { .. } = ev { true } else { false }
    }
    fn app_event(ev :&Event, app :&mut App, command_input :bool, canvas_input :bool) {
        //println!("app event {:?}");
        match ev {
            Event::TextInput { ref text, .. } => {
                for chr in text.chars() {
                    if chr == ',' {
                        if app.command_builder.is_none() {
                            app.main_menu();
                        }
                    }
                    if chr == '.' {
                        if app.command_builder.is_none() {
                            if let Some(screen) = app.context_menu() {
                                app.command_builder = Some(CommandBuilder::new_screen(screen));
                            }
                        }
                    }
                }
            }
            _ => {},
        }
        if canvas_input {
            use sdl2::keyboard::{Keycode, Mod};
            let ctrl_mod = Mod::LCTRLMOD | Mod::RCTRLMOD;
            let shift_mod = Mod::LSHIFTMOD | Mod::RSHIFTMOD;
            match ev {
                Event::KeyDown { keycode: Some(ref keycode), keymod, .. } => {
                    println!("canvas {:?}", keycode);
                    match keycode {
                        Keycode::Left | Keycode::H => {
                            if keymod.intersects(ctrl_mod) {
                                app.model.move_view(InputDir::Left);
                            } else {
                                app.model.move_selection(InputDir::Left);
                            }
                        },
                        Keycode::Right | Keycode::L => {
                            if keymod.intersects(ctrl_mod) {
                                app.model.move_view(InputDir::Right);
                            } else {
                                app.model.move_selection(InputDir::Right);
                            }
                        },
                        Keycode::Up | Keycode::K => {
                            if keymod.intersects(ctrl_mod) {
                                app.model.move_view(InputDir::Up);
                            } else {
                                app.model.move_selection(InputDir::Up);
                            }
                        },
                        Keycode::Down | Keycode::J => {
                            if keymod.intersects(ctrl_mod) {
                                app.model.move_view(InputDir::Down);
                            } else {
                                app.model.move_selection(InputDir::Down);
                            }
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }

        if command_input {
            let mut new_screen_func = None;
            if let Some(cb) = &mut app.command_builder {
                if let CommandScreen::Menu(Menu { choices }) = cb.current_screen() {
                    for (c,_,f) in choices {
                        match ev {
                            Event::TextInput { ref text, .. } => {
                                for chr in text.chars() {
                                    if chr == *c {
                                        new_screen_func = Some(*f);
                                    }
                                }
                            }
                            _ => {},
                        }
                    }
                }
            }

            if let Some(f) = new_screen_func {
                if let Some(s) = f(app) {
                    if let Some(ref mut c) = app.command_builder {
                        c.push_screen(s);
                    }
                } else {
                    app.command_builder = None;
                }
            }
        }
    }

    //let win1 = CString::new("sidebar1").unwrap();


    let mut user_data = serde_json::json!({});

    let mut open_object : OpenObject = OpenObject { 
        newkey: String::new(),
        open_subobjects: Vec::new(),
    };

    let mut sidebar_size :f32 = 200.0;
    let mut issues_size :f32 = 200.0;
    let mut graph_size :f32 = 200.0;

    let canvas_bg = 60 + (60<<8) + (60<<16) + (255<<24);
    let line_col  = 208 + (208<<8) + (175<<16) + (255<<24);
    let tvd_col  = 175 + (255<<8) + (175<<16) + (255<<24);
    let selected_col  = 175 + (175<<8) + (255<<16) + (255<<24);
    let line_hover_col  = 255 + (50<<8) + (50<<16) + (255<<24);
    //let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i :i64 = 0;
    let mut capture_command_key = false;
    let mut capture_canvas_key = false;

/*

    linux_backend_begin(|ev| {
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

    },
    || {

        */

    backend_directx9::backend(|req| {
              use self::app::*;
              use imgui_sys_bindgen::sys::*;
              let v2_0 = ImVec2 { x: 0.0, y: 0.0 };
              let small = ImVec2 { x: 200.0, y: 200.0 };

              // Check for updates from all background threads
              app.update_background_processes();

              //if let Derive::Ok(Schematic { pos_map, .. }) = &app.model.inf.schematic {
              //    println!("pos_map:  {:?}", pos_map);
              //}
                let io = unsafe { imgui_sys_bindgen::sys::igGetIO() };

              unsafe {
                  if app.show_imgui_demo {
                      igShowDemoWindow(&mut app.show_imgui_demo as *mut bool);
                  }

                  let mouse_pos = (*io).MousePos;

                  //let viewport = igGetMainViewport();
                  igSetNextWindowPos(ImVec2 { x: 0.0, y: 0.0 }, ImGuiCond__ImGuiCond_Always as _, v2_0);
                  igSetNextWindowSize((*io).DisplaySize, ImGuiCond__ImGuiCond_Always as _ );
                  let dockspace_window_flags = ImGuiWindowFlags__ImGuiWindowFlags_NoTitleBar
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoCollapse
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoResize
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoMove
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoBringToFrontOnFocus
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoNavFocus;

                  igBegin(const_cstr!("Root").as_ptr(), ptr::null_mut(), dockspace_window_flags as _ );
                  
                  let root_size = igGetContentRegionAvail_nonUDT2();
                  let root_size = ImVec2 { x: root_size.x, y: root_size.y}; 
//                  let root_size = ImVec2 { x : 500.0, y: 500.0};
                  let mut main_column_size = ImVec2 { x: root_size.x - sidebar_size, y: root_size.y };

                  igSplitter(true, 4.0, &mut sidebar_size as _, &mut main_column_size.x as _, 100.0, 100.0, -1.0);


                  //
                  // SIDEBAR
                  //
                  //
                  ui::sidebar::sidebar(ImVec2{ x: sidebar_size, y: root_size.y }, &mut app);


                  // main column
                  igSameLine(0.0, -1.0);
                  igBeginChild(const_cstr!("CanvasandIssues").as_ptr(), main_column_size, false, 0);
                  //let graph_size_open = if app.model.view.selected_dispatch.is_some() { graph_size } else { 0.0 };

                  let mut canvasgraph_size = ImVec2 { 
                      x: main_column_size.x, 
                      y: main_column_size.y - issues_size };
                  igSplitter(false, 4.0, &mut canvasgraph_size.y as _, &mut issues_size as _, 100.0, 100.0, -1.0);

                  //
                  // CANVAS
                  //
                  //

                  let graph_open = app.model.view.selected_scenario.has_dispatch();
                  let canvas_size = if graph_open {

                      let mut canvas_size_y = canvasgraph_size.y - graph_size;

                      igBeginChild(const_cstr!("canvasandgraph").as_ptr(), 
                                   canvasgraph_size, false, 0);

                      igSplitter(false, 4.0, 
                                 &mut canvas_size_y as _, 
                                 &mut graph_size,
                                 100.0, 100.0, -1.0);

                      ImVec2 {
                          x: main_column_size.x,
                          y: canvas_size_y}
                  } else {
                      canvasgraph_size
                  };

                  capture_canvas_key = ui::canvas::canvas(canvas_size, &mut app);

                  if graph_open {
                      let size = ImVec2 { x: main_column_size.x, y: graph_size };
                      // TODO capture graph key commands?
                      let capture_graph_key = ui::graph::graph(size, &mut app);

                      igEndChild();
                  }


                  //
                  // ISSUES
                  //
                  //
                   ui::issues::issues(ImVec2 { x: main_column_size.x, y: issues_size } ,&mut app);


                  igEndChild();

                  igEnd();


                  //
                  // COMMAND BUILDER 
                  // overlay window
                  //
                  //


                  capture_command_key = ui::command::command(ImVec2 { x: sidebar_size, y: 0.0 },&mut app);

              }

              true
        });


    Ok(())
}
