use backend_glfw::imgui::*;
use const_cstr::*;
use crate::{file, viewmodel, canvas, diagram, model};
use log::*;
use crate::ui;
use crate::import;

pub struct ShowWindows {
    pub config :bool,
    pub debug :bool,
    pub log :bool,
    pub vehicles :bool,
    pub quit :bool,
    pub import :import::ImportWindow,
}

pub fn main_menu(show :&mut ShowWindows,
                 doc :&mut viewmodel::ViewModel, canvas :&mut canvas::Canvas, diagram :&mut diagram::Diagram,
                 thread_pool :&threadpool::ThreadPool) {
    unsafe {
        if igBeginMenuBar() {

            if igBeginMenu(const_cstr!("File").as_ptr(), true) {

                // TODO warn about saving file when doing new file / load file
                if igMenuItemBool(const_cstr!("New file").as_ptr(), std::ptr::null(), false, true) {
                    *doc = viewmodel::ViewModel::new(model::Undoable::from(Default::default()), 
                                                     file::FileInfo::empty(), thread_pool.clone());
                    *canvas = canvas::Canvas::new();
                    *diagram = diagram::Diagram::new();
                }

                if igMenuItemBool(const_cstr!("Load file...").as_ptr(), std::ptr::null(), false, true) {
                    let mut fileinfo = doc.fileinfo.clone();
                    if let Ok(Some(m)) = file::load_doc(&mut fileinfo) {
                        info!("Loading model from file succeeded.");
                        *doc = viewmodel::ViewModel::new(model::Undoable::from(m), fileinfo, thread_pool.clone());
                        *canvas = canvas::Canvas::new();
                        *diagram = diagram::Diagram::new();
                    } else {
                        info!("Loading file failed.");
                    }
                }

                let save_label = if doc.fileinfo.filename.is_some() { const_cstr!("Save") } 
                                    else { const_cstr!("Save...") };

                if igMenuItemBool(save_label.as_ptr(), std::ptr::null(), false, true) {
                    let model = doc.get_undoable().get().clone();
                    match file::save_interactive(&mut doc.fileinfo, model) {
                        Err(x) => { error!("Could not save file: {}", x); }
                        Ok(()) => {},
                    };
                }

                if igMenuItemBool(const_cstr!("Save as...").as_ptr(), std::ptr::null(), false, true) {
                    let model = doc.get_undoable().get().clone();
                    match file::save_as_interactive(&mut doc.fileinfo, model) {
                        Err(x) => { error!("Could not save file: {}", x); }
                        Ok(()) => {},
                    };
                }

                ui::sep();

                if igMenuItemBool(const_cstr!("Import from railML...").as_ptr(), std::ptr::null(), false, true) {
                    show.import.open();
                }
                if igMenuItemBool(const_cstr!("Export to railML...").as_ptr(), std::ptr::null(), false, true) {
                }

                ui::sep();
                if igMenuItemBool(const_cstr!("Quit").as_ptr(), std::ptr::null(), false, true) {
                    show.quit = true;
                }

                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Edit").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("Edit vehicles").as_ptr(), std::ptr::null(), show.vehicles, true) {
                    show.vehicles = !show.vehicles;
                }
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("View").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("Log window").as_ptr(), std::ptr::null(), show.log, true) {
                    show.log = !show.log;
                }
                igEndMenu();
            }
            if igBeginMenu(const_cstr!("Tools").as_ptr(), true) {
                if igMenuItemBool(const_cstr!("View data").as_ptr(), std::ptr::null(), show.debug, true) {
                    show.debug = !show.debug;
                }
                if igMenuItemBool(const_cstr!("Configure colors").as_ptr(), std::ptr::null(), show.config, true) {
                    show.config = !show.config;
                }
                igEndMenu();
            }

            igEndMenuBar();
        }
    }
}

