use backend_glfw::imgui::*;
use const_cstr::*;
use log::*;

use crate::app::*;
use crate::document::Document;
use crate::file;
use crate::gui::widgets;

pub fn main_menu(app :&mut App) {
    unsafe {
        if igBeginMenuBar() {

            if igBeginMenu(const_cstr!("File").as_ptr(), true) {

                // TODO warn about saving file when doing new file / load file
                if igMenuItemBool(const_cstr!("New file").as_ptr(), std::ptr::null(), false, true) {
                    app.document = Document::empty(app.background_jobs.clone());
                }

                if igMenuItemBool(const_cstr!("Load file...").as_ptr(), std::ptr::null(), false, true) {
                    match file::load_interactive() {
                        Ok(Some((m, filename))) => {
                            info!("Loading model from file succeeded.");
                            app.document = Document::from_model(m, app.background_jobs.clone());
                            app.document.fileinfo.set_saved_file(filename);
                        },
                        Ok(None) => {
                            info!("Load file cancelled by user.");
                        },
                        Err(e) => {
                            error!("Error loading file: {}", e);
                        },
                    };
                }

                match &app.document.fileinfo.filename  {
                    Some(filename) => {
                        if igMenuItemBool(const_cstr!("Save").as_ptr(), 
                                          std::ptr::null(), false, true) {
                            match file::save(filename, app.document.viewmodel.model().clone()) {
                                Err(e) => { error!("Error saving file: {}", e); },
                                Ok(()) => { app.document.fileinfo.set_saved(); },
                            };
                        }
                    },
                    None => {
                        if igMenuItemBool(const_cstr!("Save...").as_ptr(), 
                                          std::ptr::null(), false, true) {
                            match file::save_interactive(app.document.viewmodel.model().clone()) {
                                Err(e) => { error!("Error saving file: {}", e); },
                                Ok(Some(filename)) => { app.document.fileinfo.set_saved_file(filename); },
                                _ => {}, // cancelled
                            };
                        }
                    }
                }

                if igMenuItemBool(const_cstr!("Save as...").as_ptr(), std::ptr::null(), false, true) {
                    match file::save_interactive(app.document.viewmodel.model().clone()) {
                        Err(e) => { error!("Error saving file: {}", e); },
                        _ => {},
                    }
                }

                widgets::sep();

                if igMenuItemBool(const_cstr!("Import from railML...").as_ptr(), std::ptr::null(), false, true) {
                    //show.import.open();
                    // TODO
                }

                if igMenuItemBool(const_cstr!("Export to railML...").as_ptr(), std::ptr::null(), false, true) {
                    // TODO 
                }

                widgets::sep();
                if igMenuItemBool(const_cstr!("Quit").as_ptr(), 
                                  std::ptr::null(), false, true) {
                    app.windows.quit = true;
                }

                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Edit").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("Edit vehicles").as_ptr(), 
                                  std::ptr::null(), app.windows.vehicles, true) {
                    app.windows.vehicles = !app.windows.vehicles;
                }
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("View").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("Log window").as_ptr(), 
                                  std::ptr::null(), app.windows.log, true) {
                    app.windows.log = !app.windows.log;
                }
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Tools").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("View data").as_ptr(), 
                                  std::ptr::null(), app.windows.debug, true) {
                    app.windows.debug = !app.windows.debug;
                }
                if igMenuItemBool(const_cstr!("Configure colors").as_ptr(), 
                                  std::ptr::null(), app.windows.config, true) {
                    app.windows.config = !app.windows.config;
                }
                igEndMenu();
            }

            igEndMenuBar();
        }
    }
}

