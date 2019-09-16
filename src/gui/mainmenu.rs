use nalgebra_glm as glm;
use backend_glfw::imgui::*;
use const_cstr::*;
use log::*;

use crate::app::*;
use crate::document::Document;
use crate::gui;
use crate::file;
use crate::gui::widgets;

pub fn load(app :&mut App) {
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

pub fn main_menu(app :&mut App) {
    unsafe {
        if igBeginMenuBar() {

            if igBeginMenu(const_cstr!("File").as_ptr(), true) {

                // TODO warn about saving file when doing new file / load file
                if igMenuItemBool(const_cstr!("New file").as_ptr(), std::ptr::null(), false, true) {
                    app.document = Document::empty(app.background_jobs.clone());
                    app.document.fileinfo.update_window_title();
                }

                if igMenuItemBool(const_cstr!("Save animation").as_ptr(), std::ptr::null(), false, true) {
                    saveanimation(app);
                }
                if igMenuItemBool(const_cstr!("Load file...").as_ptr(), std::ptr::null(), false, true) {

                    load(app);
                }

                match &app.document.fileinfo.filename  {
                    Some(filename) => {
                        if igMenuItemBool(const_cstr!("Save").as_ptr(), 
                                          std::ptr::null(), false, true) {
                            match file::save(filename, app.document.analysis.model().clone()) {
                                Err(e) => { error!("Error saving file: {}", e); },
                                Ok(()) => { 
                                    app.document.set_saved_file(filename.clone()); 
                                },
                            };
                        }
                    },
                    None => {
                        if igMenuItemBool(const_cstr!("Save...").as_ptr(), 
                                          std::ptr::null(), false, true) {
                            match file::save_interactive(app.document.analysis.model().clone()) {
                                Err(e) => { error!("Error saving file: {}", e); },
                                Ok(Some(filename)) => { app.document.set_saved_file(filename); },
                                _ => {}, // cancelled
                            };
                        }
                    }
                }

                if igMenuItemBool(const_cstr!("Save as...").as_ptr(), std::ptr::null(), false, true) {
                    match file::save_interactive(app.document.analysis.model().clone()) {
                        Err(e) => { error!("Error saving file: {}", e); },
                        Ok(Some(filename)) => {
                            app.document.set_saved_file(filename);
                        },
                        _ => {},
                    }
                }

                widgets::sep();

                if igMenuItemBool(const_cstr!("Import from railML...").as_ptr(), std::ptr::null(), false, true) {
                    app.windows.import_window.open = true;
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
                if igMenuItemBool(const_cstr!("Signal designer").as_ptr(), 
                                  std::ptr::null(), app.windows.synthesis_window.is_some(), true) {
                    if app.windows.synthesis_window.is_none() {
                        let model = app.document.analysis.model().clone();
                        let bg = app.background_jobs.clone();
                        app.windows.synthesis_window = 
                            Some(gui::windows::synthesis::SynthesisWindow::new(model, bg));

                    }
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


pub fn saveanimation(app :&mut App) {
    use std::fmt::Write;
    let doc = &app.document;
    let model = doc.analysis.model();
    let data = doc.analysis.data();
    let (_,dgraph) = data.dgraph.as_ref().unwrap();
    let (_,dispatch) = data.dispatch[0].as_ref().unwrap();

    let mut output = String::new();

    let total_time = dispatch.time_interval.1;
    let num_frames = 100;
    use matches::*;
    for i in -1..(num_frames+1) {
        let t = (i as f32) / ((num_frames-1) as f32) * total_time;
        use crate::document::dispatch::*;
        let instant = Instant::from(t, &dispatch.history, &dgraph);
        writeln!(output, "%% time = {}, step = {}", t, i);
        writeln!(output, "\\begin{{tikzpicture}}");
        writeln!(output, "");

        let mut min_x = std::f32::INFINITY;
        let mut max_x = -std::f32::INFINITY;
        let mut min_y = std::f32::INFINITY;
        let mut max_y = -std::f32::INFINITY;
        for l in &model.linesegs  {
            writeln!(output, "\\draw [rail] ({},{}) -- ({},{});",
                                (l.0).x,(l.0).y,(l.1).x,(l.1).y);
            min_x = min_x.min((l.0).x as f32 );
            min_x = min_x.min((l.1).x as f32 );
            max_x = max_x.max((l.0).x as f32 );
            max_x = max_x.max((l.1).x as f32 );
            min_y = min_y.min((l.0).y as f32 );
            min_y = min_y.min((l.1).y as f32 );
            max_y = max_y.max((l.0).y as f32 );
            max_y = max_y.max((l.1).y as f32 );
        }
        writeln!(output, "\\draw[draw=none,fill=none] ({},{}) -- ({},{});",
                min_x-0.5, min_y-0.5, max_x+0.5,max_y+0.5);

        for (pta,obj) in &model.objects {
            let tangent = glm::normalize(&glm::vec2(obj.tangent.x as f32, obj.tangent.y as f32));
            let normal = glm::normalize(&glm::vec2(-obj.tangent.y as f32, obj.tangent.x as f32));
            let loc = obj.loc;

            let state = instant.infrastructure.object_state.get(pta).cloned().unwrap_or(vec![]);

            use crate::document::objects::*;
            for f in obj.functions.iter() {
                match f {
                    Function::Detector => {
                        let scale = 0.1;
                        let a = loc + scale*normal;
                        let b = loc - scale*normal;
                        writeln!(output, "\\draw [detector] ({},{}) -- ({},{});",
                            a.x, a.y, b.x, b.y);
                    }
                    Function::MainSignal { .. } => {
                        let scale = 0.1;
                        // base
                        let a = loc - scale*normal;
                        let b = loc + scale*normal;
                        writeln!(output,"\\draw [signal] ({},{}) -- ({},{});", a.x, a.y, b.x, b.y);
                        // stem
                        let a = loc ;
                        let b = loc + 1.0*scale*tangent;
                        writeln!(output,"\\draw [signal] ({},{}) -- ({},{});", a.x, a.y, b.x, b.y);
                        // light
                        let green = state.iter().any(|s| matches!(s, ObjectState::SignalProceed));
                        let a = loc + 2.0*scale*tangent;
                        let rad = scale;
                        writeln!(output,"\\draw [signal,{}] ({},{}) circle ({});",
                                if green { "fill=none" } else { "fill=black" },
                                a.x, a.y, rad);

                    },
                    _ => {},
                }
            }
        }

        // trains trains
        //
        for t in instant.trains.iter() {
            use std::collections::HashSet;
            let mut lines :HashSet<((i32,i32),(i32,i32))> = t.lines.iter().map(|(p1,p2)| 
                (((p1.x*10000.0) as i32, (p1.y*10000.0) as i32),
                 ((p2.x*10000.0) as i32, (p2.y*10000.0) as i32))).collect();

            if lines.len() == 0 { continue; }

            let l1 = lines.iter().next().unwrap().clone();
            lines.remove(&l1);
            let mut current_pline = vec![l1.0,l1.1];

            while lines.len() > 0 {
                for (p1,p2) in lines.iter() {
                    if &current_pline[0] == p2 {
                        let (p1,p2) = lines.take(&(*p1,*p2)).unwrap();
                        current_pline.insert(0, p1);
                        break;
                    } else if &current_pline[0] == p1 {
                        let (p1,p2) = lines.take(&(*p1,*p2)).unwrap();
                        current_pline.insert(0, p2);
                        break;
                    } else if &current_pline[current_pline.len()-1] == p1 {
                        let (p1,p2) = lines.take(&(*p1,*p2)).unwrap();
                        current_pline.push(p2);
                        break;
                    } else if &current_pline[current_pline.len()-1] == p2 {
                        let (p1,p2) = lines.take(&(*p1,*p2)).unwrap();
                        current_pline.push(p1);
                        break;
                    } 
                }
            }

            let coords = current_pline.into_iter().map(|p| format!("({},{})", p.0 as f32 / 10000.0, 
                                                                   p.1 as f32 / 10000.0))
                .collect::<Vec<_>>();

            writeln!(output, "\\draw [train] {};", coords.join(" -- "));
        }

        writeln!(output, "\\end{{tikzpicture}}");
    }

    std::fs::write("anim.tex", output).unwrap();
}








