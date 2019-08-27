use const_cstr::*;
use crate::config::*;
use backend_glfw::imgui::*;

pub fn edit_config_window(popen :&mut bool, config :&mut Config) {
    if !*popen { return; }
    unsafe {
    igBegin(const_cstr!("Configuration").as_ptr(), popen as _, 0 as _);
    edit_config(config);
    igEnd();
    }

}

pub fn edit_config(config :&mut Config) {
    unsafe {
        for (name,color) in config.colors.iter_mut() {
            let name = COLORNAMES[name].as_ptr();
            igColorEdit4(name, &mut color.color.red as _, 0 as _);
        }
    }
}


