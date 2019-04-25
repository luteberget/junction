mod gui;
mod sdlinput;

// GUI components
mod menubar;
mod canvas;

use junc_app::App;

pub fn run(mut app :App) -> Result<(), String> {
    gui::main_loop(move |events| {
        // handle events
        for ev in events {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            match ev {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return false;
                },
                _ => {},
            }
        }


        // draw gui
        unsafe {
            use std::ptr;
            use imgui_sys_bindgen::sys::*;
            igShowDemoWindow(ptr::null_mut());

            if let Some(action) = menubar::menubar() {
                app.integrate(action);
            }

            if let Some(action) = canvas::canvas(&app) {
                app.integrate(action);
            }
        }

        return true; // Continue main loop
    })?;
    Ok(())
}

