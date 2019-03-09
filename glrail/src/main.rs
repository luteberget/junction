use std::ffi::{CString, CStr};
use std::ptr;
use const_cstr::const_cstr;
use imgui_sys_bindgen::sys::*;

mod sdlinput;

// Domain
mod app;
mod schematic;

use self::app::*;

pub fn entity_to_string(id :EntityId, inf :&Infrastructure) -> String {
  match inf.get(id) {
      Some(Entity::Track(ref t)) => {
          format!("{:#?}", t)
      },
      Some(Entity::Node(p,ref n)) => {
          format!("Id: {}", id)
      },
      Some(Entity::Object(ref o)) => {
          format!("Id: {}", id)
      },
      _ => { format!("Error id={} not found.", id) }
  }
}

use imgui_sys_bindgen::sys::ImVec2;
pub fn world2screen(topleft: ImVec2, bottomright: ImVec2, center :(f64,f64), zoom: f64, pt :(f32,f32)) -> ImVec2 {
    let scale = if bottomright.x - topleft.x < bottomright.y - topleft.y {
        (bottomright.x-topleft.x) as f64 / zoom
    } else {
        (bottomright.y-topleft.y) as f64 / zoom
    };
    let x = 0.5*(topleft.x + bottomright.x) as f64 + scale*(pt.0 as f64  - center.0);
    let y = 0.5*(topleft.y + bottomright.y) as f64 + scale*(-(pt.1 as f64 -  center.1));
    ImVec2 {x: x as _ , y: y as _ }
}

pub fn screen2worldlength(topleft: ImVec2, bottomright: ImVec2, zoom: f64, d :f32) -> f32 {
    let scale = if bottomright.x - topleft.x < bottomright.y - topleft.y {
        (bottomright.x-topleft.x) as f64 / zoom
    } else {
        (bottomright.y-topleft.y) as f64 / zoom
    };

    ((d as f64)/scale) as f32
}

pub fn  line_closest_pt(a :&ImVec2, b :&ImVec2, p :&ImVec2) -> ImVec2 {
    let ap = ImVec2{ x: p.x - a.x, y:  p.y - a.y};
    let ab_dir = ImVec2 { x: b.x - a.x, y: b.y - a.y };
    let dot = ap.x * ab_dir.x + ap.y * ab_dir.y;
    if dot < 0.0 { return *a; }
    let ab_len_sqr = ab_dir.x * ab_dir.x + ab_dir.y * ab_dir.y;
    if dot > ab_len_sqr { return *b; }
    let ac = ImVec2{ x: ab_dir.x * dot / ab_len_sqr, y: ab_dir.y * dot / ab_len_sqr } ;
    ImVec2 { x : a.x + ac.x, y: a.y + ac.y }
}

pub fn dist2(a :&ImVec2, b :&ImVec2) -> f32 { 
    (a.x - b.x)*(a.x - b.x) + (a.y - b.y)*(a.y - b.y)
}

pub struct OpenObject {
    pub newkey :String,
    pub open_subobjects :Vec<(String, Box<OpenObject>)>,
}

pub fn input_text_string(
    label: &CStr,
    hint: Option<&CStr>,
    buffer: &mut String,
    flags: ImGuiInputTextFlags) {
    buffer.push('\0');
    input_text(label,hint, unsafe { buffer.as_mut_vec() },flags);
    buffer.pop();
}

pub fn input_text(
    label: &CStr,
    hint: Option<&CStr>,
    buffer: &mut Vec<u8>,
    mut flags: ImGuiInputTextFlags) {

   unsafe extern "C" fn resize_func(data: *mut ImGuiInputTextCallbackData) -> std::os::raw::c_int  {
       //println!("BufTextLen {:?}", (*data).BufTextLen);
       let vecptr = ((*data).UserData as *mut Vec<u8>);
       (*vecptr).resize((*data).BufTextLen as usize + 1, '\0' as u8);
       (*vecptr)[(*data).BufTextLen as usize ] = '\0' as u8;
       (*data).Buf = (*vecptr).as_mut_ptr() as _;
       0
   }

   match hint {
       Some(hint) => {
           unsafe {
           igInputTextWithHint(
               label.as_ptr(),
                //const_cstr!("").as_ptr(),
                //const_cstr!("New key").as_ptr(),
               hint.as_ptr(),
                buffer.as_mut_ptr() as _,
                buffer.capacity()+1,
                flags | (ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackResize as ImGuiInputTextFlags) ,
                Some(resize_func),
                buffer as *mut _ as _);
           }
       },
       None => {
           // TODO igInputText
           unimplemented!()
       }
   }

}

pub fn show_text(s :&str) {
    unsafe {
    igTextSlice(s.as_ptr() as _ , s.as_ptr().offset(s.len() as _ ) as _ );
    }
}


type UserData = serde_json::Map<String, serde_json::Value>;

pub fn json_editor(types: &[*const i8; 6], data :&mut UserData, open :&mut OpenObject) {
    let v2_0 = ImVec2 { x: 0.0, y : 0.0 };
    unsafe {
        use imgui_sys_bindgen::sys::*;
        let mut del = None;
        for (i,(k,v)) in data.iter_mut().enumerate() {
            igPushIDInt(i as _);
            show_text(k);

            if igButton(const_cstr!("\u{f056}").as_ptr(), v2_0) {
                del = Some(k.clone());
            }
            igSameLine(0.0, -1.0);
            
            igPushItemWidth(3.0*16.0);

            let l_null = const_cstr!("null");
            let l_bool = const_cstr!("bool");
            let l_number = const_cstr!("num");
            let l_text = const_cstr!("text");
            let l_array = const_cstr!("arr");
            let l_object = const_cstr!("obj");

            let curr_type_str = match v {
                             serde_json::Value::Null => l_null,
                             serde_json::Value::Bool(_) => l_bool,
                             serde_json::Value::Number(_) => l_number,
                             serde_json::Value::String(_) => l_text,
                             serde_json::Value::Object(_) => l_object,
                             serde_json::Value::Array(_) => l_array,
                             _ => l_text,
                         };

            if igBeginCombo(const_cstr!("##type").as_ptr(), curr_type_str.as_ptr(),
                         ImGuiComboFlags__ImGuiComboFlags_NoArrowButton as _) {

                if igSelectable(l_null.as_ptr(), l_null == curr_type_str, 0, v2_0) 
                    && l_null != curr_type_str {
                        *v = serde_json::Value::Null;
                }
                if igSelectable(l_bool.as_ptr(), l_bool == curr_type_str, 0, v2_0) 
                    && l_bool != curr_type_str {
                        *v = serde_json::Value::Bool(Default::default());
                }
                if igSelectable(l_number.as_ptr(), l_number == curr_type_str, 0, v2_0) 
                    && l_number != curr_type_str {
                        *v = serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap());
                }
                if igSelectable(l_text.as_ptr(), l_text == curr_type_str, 0, v2_0) 
                    && l_text != curr_type_str {
                        *v = serde_json::Value::String(Default::default());
                }
                if igSelectable(l_array.as_ptr(), l_array == curr_type_str, 0, v2_0) 
                    && l_array != curr_type_str {
                        *v = serde_json::Value::Array(Default::default());
                }
                if igSelectable(l_object.as_ptr(), l_object == curr_type_str, 0, v2_0) 
                    && l_object != curr_type_str {
                        *v = serde_json::Value::Object(Default::default());
                }
                igEndCombo();
            }
            igPopItemWidth();

            igPushItemWidth(-1.0);

            match v {
                serde_json::Value::Null => {},
                serde_json::Value::Bool(ref mut b) => {
                    let l_true = const_cstr!("true");
                    let l_false = const_cstr!("false");
                    igSameLine(0.0, -1.0);
                    if igBeginCombo(const_cstr!("##bool").as_ptr(), 
                                    (if *b { l_true } else { l_false }).as_ptr(),0) {

                        if igSelectable(l_false.as_ptr(), !*b, 0, v2_0) && *b {
                            *b = false;
                        }
                        if igSelectable(l_true.as_ptr(), *b, 0, v2_0) && !*b {
                            *b = true;
                        }
                        igEndCombo();
                    }
                },
                serde_json::Value::Number(ref mut n) => {
                    let mut num : f32 = n.as_f64().unwrap() as _;
                    igSameLine(0.0, -1.0);
                    igInputFloat(const_cstr!("##num").as_ptr(), 
                                 &mut num as *mut _, 0.0, 1.0, 
                                 const_cstr!("%g").as_ptr(), 0);
                    if igIsItemDeactivatedAfterEdit() {
                        *n = serde_json::Number::from_f64(num as _).unwrap();
                    }
                },
                serde_json::Value::String(ref mut s) => {
                    igSameLine(0.0, -1.0);
                    input_text_string(
                        const_cstr!("##text").as_cstr(), 
                        Some(const_cstr!("empty").as_cstr()), 
                        s, 0);
                },
                serde_json::Value::Array(ref mut a) => {
                    igSameLine(0.0, -1.0);
                    if igTreeNodeStr(const_cstr!("Array").as_ptr()) {
                        igText(const_cstr!("...").as_ptr());
                        igTreePop();
                    }
                },
                serde_json::Value::Object(ref mut o) => {
                    igSameLine(0.0, -1.0);
                    if igTreeNodeStr(const_cstr!("Object").as_ptr()) {

                        //json_editor
                        json_editor(&types, o, open);

                        igTreePop();
                    }
                },
                _ => unimplemented!(),
            }

            igPopItemWidth();
            //println!("{:?}: {:?}", k,v);
            igPopID();
        }

        if let Some(k) = del {
            data.remove(&k);
        }

        if igButton(const_cstr!("\u{f055}").as_ptr(), ImVec2 { x: 0.0, y: 0.0 })  {
            use std::mem;
            let s = &mut open.newkey;
            if s.len() > 0 {
                data.insert(
                    mem::replace(s, String::new()),
                    serde_json::Value::Null);
            }
        }

       igSameLine(0.0, -1.0);
       input_text_string( const_cstr!("##newkey").as_cstr(), 
                   Some(const_cstr!("New key").as_cstr()), &mut open.newkey, 0);
    }
}



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

pub fn wake() {
    unsafe {
        use std::ptr;
        use sdl2::sys::*;

        let ev = SDL_UserEvent { 
            type_: SDL_EventType::SDL_USEREVENT as _, 
            timestamp: sdl2::sys::SDL_GetTicks(),
            windowID: 0,
            code: 0,
            data1: ptr::null_mut(),
            data2: ptr::null_mut(),
        };

        let mut ev = SDL_Event { user: ev };
        SDL_PushEvent(&mut ev as _);
    }
}

fn main() -> Result<(), String>{
    use log::LevelFilter;
    simple_logging::log_to_stderr(LevelFilter::Debug);

    let json_types: [*const i8; 6] = [
        const_cstr!("Null").as_ptr(),
        const_cstr!("Bool").as_ptr(),
        const_cstr!("Num").as_ptr(),
        const_cstr!("Text").as_ptr(),
        const_cstr!("Obj").as_ptr(),
        const_cstr!("Arr").as_ptr(),
    ];


    let mut app = app::App::new();
    //let mut action_queue = Vec::new();

    let sdl_context = sdl2::init()?;
    let event_subsystem = sdl_context.event()?;
    let video_subsystem = sdl_context.video()?;
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

        //let mut ev = SDL_Event { type_: SDL_EventType::SDL_USEREVENT as _, user: ev };
    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator : sdl2::render::TextureCreator<_> 
        = canvas.texture_creator();

    gui_init();
    let io = unsafe { imgui_sys_bindgen::sys::igGetIO() };



    unsafe {
            use imgui_sys_bindgen::sys::*;
        //    //io.Fonts->AddFontFromFileTTF("../../misc/fonts/Roboto-Medium.ttf", 16.0f);

		//      //ImVector<ImWchar> ranges;
        //    let ranges = ImVector_ImWchar_ImVector_ImWchar();
		//      //ImFontGlyphRangesBuilder builder;
        //    let builder = ImFontGlyphRangesBuilder_ImFontGlyphRangesBuilder();
        //    ImFontGlyphRangesBuilder_AddText(builder, black_left.as_ptr(), ptr::null());
        //    ImFontGlyphRangesBuilder_AddText(builder, black_right.as_ptr(), ptr::null());
        //    //
        //    //builder.AddRanges(io.Fonts->GetGlyphRangesJapanese()); // Add one of the default ranges
        //    //ImFontGlyphRangesBuilder_AddRanges( builder, ImFontAtlas_GetGlyphRangesJapanese((*io).Fonts));
        //    ImFontGlyphRangesBuilder_AddRanges( builder, ImFontAtlas_GetGlyphRangesDefault((*io).Fonts));

		//    //builder.BuildRanges(&ranges);                          // Build the final result (ordered ranges with all the unique characters submitted)
        //    ImFontGlyphRangesBuilder_BuildRanges(builder, ranges);

		//    //io.Fonts->AddFontFromFileTTF("myfontfile.ttf", size_in_pixels, NULL, ranges.Data);
		//    //io.Fonts->Build();                                     // Build the atlas while 'ranges' is still in scope and not deleted.


        //    let fconfig = ptr::null();
        //    //let franges = ptr::null();
        //    ImFontAtlas_AddFontFromFileTTF((*io).Fonts, 
        //           const_cstr!("DejaVuSansMono.ttf").as_ptr(),
        //           22.0, fconfig, (*ranges).Data);
        //    ImFontAtlas_Build((*io).Fonts);

        
        ImFontAtlas_AddFontFromFileTTF((*io).Fonts, 
               const_cstr!("DejaVuSansMono.ttf").as_ptr(),
               16.0, ptr::null(), ptr::null());

        let config = ImFontConfig_ImFontConfig();
        (*config).MergeMode = true;
        (*config).GlyphMinAdvanceX = 16.0;
        let ranges : [std::os::raw::c_ushort;3] = [0xf000, 0xf82f, 0x0];
        //#define ICON_MIN_FA 0xf000
        //#define ICON_MAX_FA 0xf82f

        ImFontAtlas_AddFontFromFileTTF((*io).Fonts,
            const_cstr!("fa-solid-900.ttf").as_ptr(),
            16.0,  config, &ranges as _ );

        ImFontAtlas_Build((*io).Fonts);
    }

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
        use imgui_sys_bindgen::sys::*;
        //(*imgui_sys_bindgen::sys::igGetIO()).IniFilename = ptr::null_mut();
        (*igGetIO()).ConfigFlags |= ImGuiConfigFlags__ImGuiConfigFlags_NavEnableKeyboard as i32;
    }

    let mut user_data = serde_json::json!({});

    let mut open_object : OpenObject = OpenObject { 
        newkey: String::new(),
        open_subobjects: Vec::new(),
    };

    let mut sidebar_size :f32 = 200.0;
    let mut issues_size :f32 = 200.0;
    let canvas_bg = 60 + (60<<8) + (60<<16) + (255<<24);
    let line_col  = 208 + (208<<8) + (175<<16) + (255<<24);
    let line_hover_col  = 255 + (50<<8) + (50<<16) + (255<<24);
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
              let small = ImVec2 { x: 200.0, y: 200.0 };

              // Check for updates from all background threads
              app.update();

              unsafe {
                  igShowDemoWindow(ptr::null_mut());

                  let mouse_pos = (*io).MousePos;

                  let viewport = igGetMainViewport();
                  igSetNextWindowPos((*viewport).Pos, ImGuiCond__ImGuiCond_Always as _, v2_0);
                  igSetNextWindowSize((*viewport).Size, ImGuiCond__ImGuiCond_Always as _ );
                  let dockspace_window_flags = ImGuiWindowFlags__ImGuiWindowFlags_NoTitleBar
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoCollapse
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoResize
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoMove
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoBringToFrontOnFocus
                      | ImGuiWindowFlags__ImGuiWindowFlags_NoNavFocus;

                  igBegin(const_cstr!("Root").as_ptr(), ptr::null_mut(), dockspace_window_flags as _ );
                  
                  let mut root_size = igGetContentRegionAvail();
                  let mut main_size = ImVec2 { x: root_size.x - sidebar_size, ..root_size };

                  igSplitter(true, 2.0, &mut sidebar_size as _, &mut main_size.x as _, 100.0, 100.0, -1.0);

                  igBeginChild(const_cstr!("Sidebar").as_ptr(), ImVec2 { x: sidebar_size, y: root_size.y } , false,0);
                  
                  if igCollapsingHeader(const_cstr!("All objects").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      for (i,e) in app.model.inf.entities.iter().enumerate() {
                          match e {
                              Some(Entity::Track(_))  => { 
                                  let s = CString::new(format!("Track##{}", i)).unwrap();
                                  if igSelectable(s.as_ptr(),
                                                  app.view.selected_object == Some(i), 0, v2_0) {
                                      //println!("SET {}", i);
                                      app.view.selected_object = Some(i);
                                  }
                              },
                              Some(Entity::Node(p,_))   => { 
                                  let s = CString::new(format!("Node @ {}##{}", p,i)).unwrap();
                                  if igSelectable(s.as_ptr(), 
                    
                              app.view.selected_object == Some(i), 0, v2_0) {
                                      //println!("SET NODE {}", i);
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
                          Some(id) => {
                              let s = entity_to_string(*id, &app.model.inf);
                              show_text(&s);
                          },
                          None => {
                              igText(const_cstr!("No object selected.").as_ptr());
                          }
                      }
                  }

                  if igButton(const_cstr!("Add track").as_ptr(), ImVec2 {x:  0.0, y: 0.0 }) {
                      app.integrate(EditorAction::Inf(
                              InfrastructureEdit::NewTrack(0.0,100.0)));
                  }

                  pub fn middle_of_track(model :&Model, obj :Option<EntityId>) -> Option<(EntityId, f32)> {
                      let id = obj?;
                      let Track { ref start_node, ref end_node, .. } = model.inf.get_track(id)?;
                      let (p1,_) = model.inf.get_node(start_node.0)?;
                      let (p2,_) = model.inf.get_node(end_node.0)?;
                      Some((id, 0.5*(p1+p2)))
                  }

                  if igButton(const_cstr!("Add up left switch").as_ptr(), ImVec2 {x:  0.0, y: 0.0 }) {
                      if let Some((curr_track, curr_pos)) = middle_of_track(&app.model, app.view.selected_object) {
                          app.integrate(EditorAction::Inf(
                                  InfrastructureEdit::InsertNode(
                                      curr_track, curr_pos, Node::Switch(Dir::Up, Side::Left), 50.0)));
                      } else {
                          println!("Track not selected.");
                      }
                  }

                  if igButton(const_cstr!("Load").as_ptr(), ImVec2 {x:  0.0, y: 0.0 }) {
                      sdl2::messagebox::show_simple_message_box(
                          sdl2::messagebox::MessageBoxFlag::empty(),
                          "Load file", "Load file?", canvas.window());
                  }
                  if igButton(const_cstr!("Quit").as_ptr(), ImVec2 {x:  0.0, y: 0.0 }) {
                      sdl2::messagebox::show_simple_message_box(
                          sdl2::messagebox::MessageBoxFlag::empty(),
                          "Quit", "Quit", canvas.window());
                      break 'running;
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

                  if igCollapsingHeader(const_cstr!("User data editor").as_ptr(),
                                        ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen as _ ) {
                      json_editor(&json_types, user_data.as_object_mut().unwrap(), &mut open_object);
                  }

                  igEndChild();
                  igSameLine(0.0, -1.0);
                  igBeginChild(const_cstr!("CanvasandIssues").as_ptr(), main_size, false, 0);

                  let mut mainmain_size = ImVec2 { y: main_size.y - issues_size, ..main_size };
                  igSplitter(false, 2.0, &mut mainmain_size.y as _, &mut issues_size as _, 100.0, 100.0, -1.0);

                  // CANVAS!

                  igBeginChild(const_cstr!("Canvas").as_ptr(), mainmain_size, false, 0);
                  let draw_list = igGetWindowDrawList();
                  igText(const_cstr!("Here is the canvas:").as_ptr());

                  match &app.model.inf.schematic {
                      Derive::Wait => {
                          igText(const_cstr!("Solving...").as_ptr());
                      },
                      Derive::Error(ref e) => {
                          let s = CString::new(format!("Error: {}", e)).unwrap();
                          igText(s.as_ptr());
                      },
                      Derive::Ok(ref s) => {
                          let mut hovered_item = None;
                          let canvas_pos = igGetCursorScreenPos();
                          let mut canvas_size = igGetContentRegionAvail();
                          let canvas_lower = ImVec2 { x: canvas_pos.x + canvas_size.x,
                                                      y: canvas_pos.y + canvas_size.y };
                          if canvas_size.x < 10.0 { canvas_size.x = 10.0 }

                          if canvas_size.y < 10.0 { canvas_size.y = 10.0 }
                          ImDrawList_AddRectFilled(draw_list, canvas_pos,
                                                   ImVec2 { x: canvas_pos.x + canvas_size.x,
                                                            y: canvas_pos.y + canvas_size.y, },
                                                            canvas_bg,
                                                    0.0, 0);
                          let clicked = igInvisibleButton(const_cstr!("canvasbtn").as_ptr(), canvas_size);

                          let (center,zoom) = app.view.viewport;

                          if igIsItemActive() && igIsMouseDragging(0,-1.0) {
                              (app.view.viewport.0).0 -= screen2worldlength(canvas_pos, canvas_lower, zoom, (*io).MouseDelta.x) as f64;
                              (app.view.viewport.0).1 += screen2worldlength(canvas_pos, canvas_lower, zoom, (*io).MouseDelta.y) as f64;
                          }

                          if igIsItemHovered(0) {
                              let wheel = (*io).MouseWheel;
                              //println!("{}", wheel);
                              let wheel2 = 1.0-0.2*(*io).MouseWheel;
                              //println!("{}", wheel2);
                              (app.view.viewport.1) *= wheel2 as f64;
                          }
                          

                          // Iterate the schematic 


                          ImDrawList_PushClipRect(draw_list, canvas_pos, canvas_lower, true);

                          for (k,v) in &s.lines {
                              //println!("{:?}, {:?}", k,v);
                              let mut hovered = false;
                              for i in 0..(v.len()-1) {
                                  let p1 = world2screen(canvas_pos, canvas_lower, center, zoom, v[i]);
                                  let p2 = world2screen(canvas_pos, canvas_lower, center, zoom, v[i+1]);
                                  let hovered = dist2(&mouse_pos, &line_closest_pt(&p1, &p2, &mouse_pos)) < 100.0;
                                  if hovered {
                                      hovered_item = Some(*k);
                                  }
                                  ImDrawList_AddLine(draw_list, p1, p2, 
                                                     if hovered { line_hover_col } else { line_col }, 2.0);
                              }
                          }
                          for (k,v) in &s.points {
                              let p = world2screen(canvas_pos, canvas_lower, center, zoom, *v);
                              let caret_right = const_cstr!("\u{f0da}");
                              ImDrawList_AddText(draw_list, p, line_col, caret_right.as_ptr(), ptr::null());
                          }

                          ImDrawList_PopClipRect(draw_list);

                          if let Some(id) = hovered_item {
                              if clicked {
                                  app.view.selected_object = Some(id);
                              }
                              igBeginTooltip();
                              show_text(&entity_to_string(id, &app.model.inf));
                              igEndTooltip();
                          }

                      },
                  }

                  igEndChild();


                  igBeginChild(const_cstr!("Issues").as_ptr(),ImVec2 { x: main_size.x, y: issues_size } ,false,0);
                  igText(const_cstr!("Here are the issues:").as_ptr());
                  for error in &app.model.errors {

                  }
                  igEndChild();



                  igEndChild();



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
