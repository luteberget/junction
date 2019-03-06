
use sdl2::sys as sdl2_sys;

use sdl2::video::Window;
use sdl2::mouse::{Cursor,SystemCursor,MouseState};
//use imgui::{ImGui,ImGuiMouseCursor};
use std::time::Instant;
use std::os::raw::{c_char, c_void};

use sdl2::event::Event;

pub struct ImguiSdl2 {
  last_frame: Instant,
  mouse_press: [bool; 5],
  ignore_mouse: bool,
  ignore_keyboard: bool,
  //cursor: (ImGuiMouseCursor, Option<Cursor>),
}

impl ImguiSdl2 {
  pub fn new(
    //imgui: &mut ImGui,
  ) -> Self {
    // TODO: upstream to imgui-rs
   // {
   unsafe {
       use std::ptr;
       let io = igGetIO();
        use imgui_sys_bindgen::sys::*;
        (*io).GetClipboardTextFn = Some(get_clipboard_text);
        (*io).SetClipboardTextFn = Some(set_clipboard_text);
        (*io).ClipboardUserData = ptr::null_mut();


        use sdl2::keyboard::Scancode;
        //*((*io).KeyMap.offset(ImGuiKey__ImGuiKey_LeftArrow)) = Scancode::Left as u8;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Tab		    as usize ]) =  Scancode::Tab as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_LeftArrow	as usize ]) =  Scancode::Left as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_RightArrow	as usize ]) =  Scancode::Right as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_UpArrow	    as usize ]) =  Scancode::Up as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_DownArrow	as usize ]) =  Scancode::Down as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_PageUp		as usize ]) =  Scancode::PageUp as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_PageDown	    as usize ]) =  Scancode::PageDown as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Home		    as usize ]) =  Scancode::Home as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_End		    as usize ]) =  Scancode::End as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Delete		as usize ]) =  Scancode::Delete as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Backspace	as usize ]) =  Scancode::Backspace as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Enter		as usize ]) =  Scancode::Return as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Escape		as usize ]) =  Scancode::Escape as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_A		    as usize ]) =  Scancode::A as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_C		    as usize ]) =  Scancode::C as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_V		    as usize ]) =  Scancode::V as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_X		    as usize ]) =  Scancode::X as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Y		    as usize ]) =  Scancode::Y as i32;
        ((*io).KeyMap[ ImGuiKey__ImGuiKey_Z		    as usize ]) =  Scancode::Z as i32;

   }
   //   let io = unsafe { &mut *imgui_sys::igGetIO() };

   //   io.get_clipboard_text_fn = Some(get_clipboard_text);
   //   io.set_clipboard_text_fn = Some(set_clipboard_text);
   //   io.clipboard_user_data = std::ptr::null_mut();
   // }

   // {
   //   use sdl2::keyboard::Scancode;
   //   use imgui::ImGuiKey;

   //   imgui.set_imgui_key(ImGuiKey::Tab, Scancode::Tab as u8);
   //   imgui.set_imgui_key(ImGuiKey::LeftArrow, Scancode::Left as u8);
   //   imgui.set_imgui_key(ImGuiKey::RightArrow, Scancode::Right as u8);
   //   imgui.set_imgui_key(ImGuiKey::UpArrow, Scancode::Up as u8);
   //   imgui.set_imgui_key(ImGuiKey::DownArrow, Scancode::Down as u8);
   //   imgui.set_imgui_key(ImGuiKey::PageUp, Scancode::PageUp as u8);
   //   imgui.set_imgui_key(ImGuiKey::PageDown, Scancode::PageDown as u8);
   //   imgui.set_imgui_key(ImGuiKey::Home, Scancode::Home as u8);
   //   imgui.set_imgui_key(ImGuiKey::End, Scancode::End as u8);
   //   imgui.set_imgui_key(ImGuiKey::Delete, Scancode::Delete as u8);
   //   imgui.set_imgui_key(ImGuiKey::Backspace, Scancode::Backspace as u8);
   //   imgui.set_imgui_key(ImGuiKey::Enter, Scancode::Return as u8);
   //   imgui.set_imgui_key(ImGuiKey::Escape, Scancode::Escape as u8);
   //   imgui.set_imgui_key(ImGuiKey::A, Scancode::A as u8);
   //   imgui.set_imgui_key(ImGuiKey::C, Scancode::C as u8);
   //   imgui.set_imgui_key(ImGuiKey::V, Scancode::V as u8);
   //   imgui.set_imgui_key(ImGuiKey::X, Scancode::X as u8);
   //   imgui.set_imgui_key(ImGuiKey::Y, Scancode::Y as u8);
   //   imgui.set_imgui_key(ImGuiKey::Z, Scancode::Z as u8);
   // }

    Self {
      last_frame: Instant::now(),
      mouse_press: [false; 5],
      ignore_keyboard: false,
      ignore_mouse: false,
      //cursor: (ImGuiMouseCursor::None, None),
    }
  }

  pub fn ignore_event(
    &self,
    event: &Event,
  ) -> bool {
    match *event {
      Event::KeyDown{..}
        | Event::KeyUp{..}
        | Event::TextEditing{..}
        | Event::TextInput{..}
        => self.ignore_keyboard,
      Event::MouseMotion{..}
        | Event::MouseButtonDown{..}
        | Event::MouseButtonUp{..}
        | Event::MouseWheel{..}
        | Event::FingerDown{..}
        | Event::FingerUp{..}
        | Event::FingerMotion{..}
        | Event::DollarGesture{..}
        | Event::DollarRecord{..}
        | Event::MultiGesture{..}
        => self.ignore_mouse,
      _ => false,
    }
  }

  pub fn handle_event(
    &mut self,
    //imgui: &mut ImGui,
    event: &Event,
  ) {
      use imgui_sys_bindgen::sys::*;
    use sdl2::mouse::MouseButton;
    use sdl2::keyboard;

    unsafe fn set_mod(io : *mut ImGuiIO, keymod: keyboard::Mod) {
      let ctrl = keymod.intersects(keyboard::Mod::RCTRLMOD | keyboard::Mod::LCTRLMOD);
      let alt = keymod.intersects(keyboard::Mod::RALTMOD | keyboard::Mod::LALTMOD);
      let shift = keymod.intersects(keyboard::Mod::RSHIFTMOD | keyboard::Mod::LSHIFTMOD);
      let super_ = keymod.intersects(keyboard::Mod::RGUIMOD | keyboard::Mod::LGUIMOD);

      (*io).KeyCtrl = ctrl;
      (*io).KeyAlt = alt;
      (*io).KeyShift = shift;
      (*io).KeySuper = super_;
    }

    let io = unsafe { igGetIO() };

    match *event {
      Event::MouseWheel{y, ..} => {
        unsafe { (*io).MouseWheel = y as f32; }
      },
      Event::MouseButtonDown{mouse_btn, ..} => {
        if mouse_btn != MouseButton::Unknown {
          let index = match mouse_btn {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::X1 => 3,
            MouseButton::X2 => 4,
            MouseButton::Unknown => unreachable!(),
          };
          self.mouse_press[index] = true;
        }
      },
      Event::TextInput{ref text, .. } => {
        for chr in text.chars() {
          //imgui.add_input_character(chr);
          let mut buf = [0;5];
          chr.encode_utf8(&mut buf);
          unsafe {
              ImGuiIO_AddInputCharactersUTF8(io, buf.as_ptr() as _);
          }

        }
      },
      Event::KeyDown{scancode, keymod, .. } => {
        unsafe {set_mod(io, keymod); }
        if let Some(scancode) = scancode {
          //imgui.set_key(scancode as u8, true);
          unsafe { (*io).KeysDown[scancode as usize] = true; }
        }
      },
      Event::KeyUp{scancode, keymod, .. } => {
        unsafe {set_mod(io, keymod); }
        if let Some(scancode) = scancode {
          //imgui.set_key(scancode as u8, false);
          unsafe { (*io).KeysDown[scancode as usize] = false; }
        }
      },
      _ => {},
    }
  }

  pub fn frame(
    &mut self,
    window: &Window,
    //imgui: &'ui mut ImGui,
    mouse_state: &MouseState,
  ) {
      use imgui_sys_bindgen::sys::*;
    let io = unsafe { igGetIO() };

    let mouse_util = window.subsystem().sdl().mouse();

    // Merging the mousedown events we received into the current state prevents us from missing
    // clicks that happen faster than a frame
    let mouse_down = [
      self.mouse_press[0] || mouse_state.left(),
      self.mouse_press[1] || mouse_state.right(),
      self.mouse_press[2] || mouse_state.middle(),
      self.mouse_press[3] || mouse_state.x1(),
      self.mouse_press[4] || mouse_state.x2(),
    ];
    //imgui.set_mouse_down(mouse_down);
    
    unsafe {
    (*io).MouseDown = mouse_down;
    }

    self.mouse_press = [false; 5];

    let any_mouse_down = mouse_down.iter().any(|&b| b);
    mouse_util.capture(any_mouse_down);


    //imgui.set_mouse_pos(mouse_state.x() as f32, mouse_state.y() as f32);

    unsafe {
    (*io).MousePos.x = mouse_state.x() as f32;
    (*io).MousePos.y = mouse_state.y() as f32;
    }



    //let mouse_cursor = imgui.mouse_cursor();
    //if imgui.mouse_draw_cursor() || mouse_cursor == ImGuiMouseCursor::None {
    //  self.cursor = (ImGuiMouseCursor::None, None);
    //  mouse_util.show_cursor(false);
    //} else {
    //  mouse_util.show_cursor(true);

    //  if mouse_cursor != self.cursor.0 {
    //    let sdl_cursor = match mouse_cursor {
    //      ImGuiMouseCursor::None => unreachable!("mouse_cursor was None!"),
    //      ImGuiMouseCursor::Arrow => SystemCursor::Arrow,
    //      ImGuiMouseCursor::TextInput => SystemCursor::IBeam,
    //      ImGuiMouseCursor::ResizeAll => SystemCursor::SizeAll,
    //      ImGuiMouseCursor::ResizeNS => SystemCursor::SizeNS,
    //      ImGuiMouseCursor::ResizeEW => SystemCursor::SizeWE,
    //      ImGuiMouseCursor::ResizeNESW => SystemCursor::SizeNESW,
    //      ImGuiMouseCursor::ResizeNWSE => SystemCursor::SizeNWSE,
    //      ImGuiMouseCursor::Hand => SystemCursor::Hand,
    //    };

    //    let sdl_cursor = Cursor::from_system(sdl_cursor).unwrap();
    //    sdl_cursor.set();

    //    self.cursor = (mouse_cursor, Some(sdl_cursor));
    //  }
    //}




    let now = Instant::now();
    let delta = now - self.last_frame;
    let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
    self.last_frame = now;

    let window_size = window.size();
    let display_size = window.drawable_size();

    //let frame_size = imgui::FrameSize{
    //  logical_size: (window_size.0 as f64, window_size.1 as f64),
    //  hidpi_factor: (display_size.0 as f64) / (window_size.0 as f64),
    //};
    //let ui = imgui.frame(frame_size, delta_s);

    unsafe {
        use imgui_sys_bindgen::sys::*;

        let io = igGetIO();
        (*io).DisplaySize.x = window_size.0 as f32;
        (*io).DisplaySize.y = window_size.1 as f32;
        (*io).DisplayFramebufferScale.x = (display_size.0 as f32) / (window_size.0 as f32);
        (*io).DisplayFramebufferScale.y = (display_size.0 as f32) / (window_size.0 as f32);
        (*io).DeltaTime = delta_s;

        igNewFrame();
    }

    //self.ignore_keyboard = ui.want_capture_keyboard();
    //self.ignore_mouse = ui.want_capture_mouse();

    //ui
  }
}

#[doc(hidden)]
pub extern "C" fn get_clipboard_text(_user_data: *mut c_void) -> *const c_char {
  unsafe { sdl2_sys::SDL_GetClipboardText() }
}

#[doc(hidden)]
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]
pub extern "C" fn set_clipboard_text(_user_data: *mut c_void, text: *const c_char) {
  unsafe { sdl2_sys::SDL_SetClipboardText(text) };
}
