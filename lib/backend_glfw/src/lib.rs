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

}

pub enum SystemAction {
    Draw,
    Close,
}

extern "C" { 
    fn glfw_opengl3_Init(window_name :*const i8);  
    fn glfw_opengl3_StartFrame();  
    fn glfw_opengl3_EndFrame(); 
    fn glfw_opengl3_HandleEvents(close :*mut bool); 
    fn glfw_opengl3_Destroy(); 
}

pub fn backend(window_name :&str, 
               mut handle :impl FnMut(SystemAction) -> bool) -> Result<(), String> {
    let window_name = std::ffi::CString::new(window_name).map_err(|e| format!("{:?}", e))?;
    unsafe { glfw_opengl3_Init(window_name.as_ptr()); } // Extern call to modified imgui example code.
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
        backend("imgui test", |a| { 
            unsafe { imgui::igShowDemoWindow(std::ptr::null_mut()); }
            if let SystemAction::Close = a { false } else { true }
        });
    }
}

