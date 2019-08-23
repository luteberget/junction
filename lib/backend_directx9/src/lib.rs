
pub enum SystemAction {
    Draw,
}

#[link(name = "d3d9")]
extern {}

extern "C" { 
    fn Example_Win32_DirectX9_Init();  
    fn Example_Win32_DirectX9_StartFrame();  
    fn Example_Win32_DirectX9_EndFrame(); 
    fn Example_Win32_DirectX9_HandleEvents(); 
    fn Example_Win32_DirectX9_Destroy(); 
}

pub fn backend(handle :impl FnMut(SystemAction) -> bool) -> Result<(), String> {
    windows_backend(handle)
}

pub fn windows_backend(mut handle :impl FnMut(SystemAction) -> bool) -> Result<(), String> {
    unsafe { Example_Win32_DirectX9_Init(); } // Extern call to modified imgui example code.
    loop {
        unsafe { Example_Win32_DirectX9_HandleEvents(); }
        unsafe { Example_Win32_DirectX9_StartFrame(); }
            if !handle(SystemAction::Draw) { break; }
        unsafe { Example_Win32_DirectX9_EndFrame(); }
    }
    unsafe { Example_Win32_DirectX9_Destroy(); } // Extern call to modified imgui example code.
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::*;
        windows_backend(|_| { true });
    }
}
