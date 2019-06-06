
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
        backend(|_| { true });
    }
}
