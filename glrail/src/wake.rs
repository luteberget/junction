
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
