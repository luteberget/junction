use const_cstr::*;
use crate::config::*;
use backend_glfw::imgui::*;
use log::*;

use crate::gui::widgets;

pub fn edit_config_window(popen :&mut bool, config :&mut Config) {
    if !*popen { return; }
    unsafe {
        let win_flags = ImGuiWindowFlags__ImGuiWindowFlags_MenuBar;
        if igBegin(const_cstr!("Configuration").as_ptr(), popen as _, win_flags as _) {

            if igBeginMenuBar() {
                if igBeginMenu(const_cstr!("Load").as_ptr(), true) {
                    if igMenuItemBool(const_cstr!("Save configuration").as_ptr(), std::ptr::null(), false, true) {
                        config.save();
                    }
                    if igMenuItemBool(const_cstr!("Revert to saved configuration").as_ptr(), std::ptr::null(), false, true) {
                        *config = Config::load();
                    }
                    igEndMenu();
                }
                if igBeginMenu(const_cstr!("Themes").as_ptr(), true) {
                    if igMenuItemBool(const_cstr!("Junction-gray").as_ptr(), std::ptr::null(), false, true) {
                        let s = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/gray.toml"));
                        import_string(config, s);
                    }
                    if igMenuItemBool(const_cstr!("Junction-dark").as_ptr(), std::ptr::null(), false, true) {
                        let s = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/dark.toml"));
                        import_string(config, s);
                    }
                    if igMenuItemBool(const_cstr!("Junction-light").as_ptr(), std::ptr::null(), false, true) {
                        let s = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/themes/light.toml"));
                        import_string(config, s);
                    }

                    widgets::sep();

                    if igMenuItemBool(const_cstr!("Import configuration...").as_ptr(), std::ptr::null(), false, true) {
                        if let Err(e) = import(config) {
                            error!("Could not import configuration: {}", e);
                        }
                    }
                    if igMenuItemBool(const_cstr!("Export configuration...").as_ptr(), std::ptr::null(), false, true) {
                        if let Err(e) = export(config) {
                            error!("Could not export configuration: {}", e);
                        }
                    }

                    igEndMenu();
                }
                igEndMenuBar();
            }

            edit_config(config);
        }
        igEnd();
    }
}

fn import_string(config :&mut Config, s: &str) -> Result<(), ()> {
    let data : ConfigString = toml::from_str(s).map_err(|e| ())?;
    *config = Config::from_config_string(&data);
    Ok(())
}

fn import(config :&mut Config) -> Result<(), std::io::Error> {
    if let Some(filename) = tinyfiledialogs::open_file_dialog("Import Junction config file", "",
                                             Some((&["*.toml"],"TOML files"))) {
        let data : ConfigString = toml::from_str(&std::fs::read_to_string(filename)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, "TOML conversion error"))?;
        *config = Config::from_config_string(&data);
    }
    Ok(())
}

fn export(config :&Config) -> Result<(), std::io::Error> {
    if let Some(filename) = tinyfiledialogs::save_file_dialog("Export Junction config file","") {
        let data = toml::to_string(&config.to_config_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, "TOML conversion error"))?;
        std::fs::write(filename,data)?;
    }
    Ok(())
}


pub fn edit_config(config :&mut Config) {
    unsafe {
        for (name,color) in config.colors.iter_mut() {
            let name = COLORNAMES[name].as_ptr();
            igColorEdit4(name, &mut color.color.red as _, 0 as _);
        }

        widgets::sep();

        igPushIDInt(9123 as _);
        igShowStyleEditor(std::ptr::null_mut());
        igPopID();
    }
}


