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

}

pub enum SystemAction {
    Draw,
}

extern "C" { 
    fn glfw_opengl3_Init();  
    fn glfw_opengl3_StartFrame();  
    fn glfw_opengl3_EndFrame(); 
    fn glfw_opengl3_HandleEvents(); 
    fn glfw_opengl3_Destroy(); 
}

pub fn backend(mut handle :impl FnMut(SystemAction) -> bool) -> Result<(), String> {
    unsafe { glfw_opengl3_Init(); } // Extern call to modified imgui example code.
    loop {
        unsafe { glfw_opengl3_HandleEvents(); }
        unsafe { glfw_opengl3_StartFrame(); }
            if !handle(SystemAction::Draw) { break; }
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
        backend(|_| { 
            unsafe { imgui::igShowDemoWindow(std::ptr::null_mut()); }
            true 
        });
    }
}

