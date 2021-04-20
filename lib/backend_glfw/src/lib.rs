
pub mod fa_icons;


pub mod imgui {
	#![allow(non_upper_case_globals)]
	#![allow(non_camel_case_types)]
	#![allow(non_snake_case)]
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    impl From<ImVec2_Simple> for ImVec2 {
        fn from(v :ImVec2_Simple) -> ImVec2 {
            ImVec2 { x: v.x, y: v.y }
        }
    }

    impl std::ops::Add for ImVec2 {
        type Output=Self;
        fn add(self, other :Self) -> Self {
            Self { x: self.x + other.x, y: self.y + other.y }
        }
    }

    impl std::ops::Sub for ImVec2 {
        type Output=Self;
        fn sub(self, other :Self) -> Self {
            Self { x: self.x - other.x, y: self.y - other.y }
        }
    }

    impl std::ops::Mul<ImVec2> for f32 {
        type Output = ImVec2;
        fn mul(self, v :ImVec2) -> ImVec2 {
            ImVec2 { x: v.x * self, y: v.y * self }
        }
    }

    impl std::ops::Div<f32> for ImVec2 {
        type Output = ImVec2;
        fn div(self, d :f32) -> ImVec2 {
            ImVec2 { x: self.x / d, y: self.y / d }
        }
    }

    impl ImVec2 {
        pub fn length(&self) -> f32 {
            (self.x*self.x+self.y*self.y).sqrt()
        }
        pub fn length_sq(&self) -> f32 {
            self.x*self.x+self.y*self.y
        }
        pub fn zero() -> ImVec2 {
            ImVec2 { x: 0.0, y: 0.0 }
        }
    }

}

pub enum SystemAction {
    Draw,
    Close,
}

extern "C" { 
    fn glfw_opengl3_Init(window_name :*const i8, font_name :*const i8, font_size :f32, fontawesome_ttf :*const i8, fontawesome_len :u32);  
    fn glfw_opengl3_StartFrame();  
    fn glfw_opengl3_EndFrame(); 
    fn glfw_opengl3_HandleEvents(close :*mut bool); 
    fn glfw_opengl3_Destroy(); 
    fn glfw_opengl3_SetWindowTitle(name :*const i8);
    fn glfw_opengl3_Screenshot(filename :*const i8, width :u32, height :u32);
}

pub fn set_window_title(name :&str) {
    use std::ffi::CString;
    let c_string = CString::new(name).unwrap();
    unsafe { glfw_opengl3_SetWindowTitle(c_string.as_ptr()); }
}

pub fn screenshot(filename :&str, 
                  font_name :Option<&str>,
                  font_size :f32,
                  mut f :impl FnMut()) -> Result<(), String> {

    let fontawesome_ttf = fa_icons::FA_TTF_CONTENTS;

    let window_name = std::ffi::CString::new("glfw").map_err(|e| format!("{:?}", e))?;
    let font_name_string = {
        match font_name {
            Some(name) => Some(std::ffi::CString::new(name).map_err(|e| format!("{:?}", e))?),
            None => None,
        }
    };

    // Extern call to modified imgui example code.
    unsafe { glfw_opengl3_Init(window_name.as_ptr(), 
                   font_name_string.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                   font_size,
                   fontawesome_ttf.as_ptr() as _ , fontawesome_ttf.len() as _ ); }

    let mut close = false;
    unsafe { glfw_opengl3_HandleEvents(&mut close as *mut _); }
    unsafe { glfw_opengl3_StartFrame(); }

    f();

    let width = unsafe { (*(imgui::igGetIO())).DisplaySize.x.round() as _ };
    let height = unsafe { (*(imgui::igGetIO())).DisplaySize.y.round() as _ };
    unsafe { glfw_opengl3_EndFrame(); }

    unsafe { glfw_opengl3_StartFrame(); }

    let name = std::ffi::CString::new(filename).unwrap();
    unsafe {
        glfw_opengl3_Screenshot(
            name.as_ptr(),
            width, height);
    }

    unsafe { glfw_opengl3_EndFrame(); }

    unsafe { glfw_opengl3_Destroy(); } // Extern call to modified imgui example code.
    Ok(())

}

pub fn backend(window_name :&str, 
               font_name :Option<&str>,
               font_size :f32,
               mut handle :impl FnMut(SystemAction) -> bool) -> Result<(), String> {

    let fontawesome_ttf = fa_icons::FA_TTF_CONTENTS;

    let window_name = std::ffi::CString::new(window_name).map_err(|e| format!("{:?}", e))?;
    let font_name_string = {
        match font_name {
            Some(name) => Some(std::ffi::CString::new(name).map_err(|e| format!("{:?}", e))?),
            None => None,
        }
    };

    // Extern call to modified imgui example code.
    unsafe { glfw_opengl3_Init(window_name.as_ptr(), 
                   font_name_string.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                   font_size,
                   fontawesome_ttf.as_ptr() as _ , fontawesome_ttf.len() as _ ); }

    loop {
        let mut close = false;
        unsafe { glfw_opengl3_HandleEvents(&mut close as *mut _); }
        unsafe { glfw_opengl3_StartFrame(); }
        let action = match close {
            false => SystemAction::Draw,
            true => SystemAction::Close,
        };
        if !handle(action) { break; }
        unsafe { glfw_opengl3_EndFrame(); }
    }
    unsafe { glfw_opengl3_Destroy(); } // Extern call to modified imgui example code.
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::*;
        backend("imgui test",None, 14.0, |a| { 
            unsafe { imgui::igShowDemoWindow(std::ptr::null_mut()); }
            if let SystemAction::Close = a { false } else { true }
        });
    }
}

