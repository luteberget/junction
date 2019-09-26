use crate::app::App;
use crate::document::objects::*;
use crate::document::infview::*;
use crate::gui::mainmenu;
use crate::file;
use crate::document::*;

use log::*;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;

pub fn keys(app :&mut App) {
    unsafe {
        let io = igGetIO();


        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            app.document.analysis.undo();
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            app.document.analysis.redo();
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Y' as _, false) {
            app.document.analysis.redo();
        }

        if (*io).KeyCtrl && igIsKeyPressed('S' as _, false) {
            match (&app.document.fileinfo.filename, (*io).KeyShift) {
                (None,_) | (_,true) => {
                    match file::save_interactive(app.document.analysis.model().clone()) {
                        Err(e) => { error!("Error saving file: {}", e); },
                        Ok(Some(filename)) => { app.document.set_saved_file(filename); },
                        _ => {},
                    }
                }
                (Some(filename),_) => {
                    match file::save(filename, app.document.analysis.model().clone()) {
                        Err(e) => { error!("Error saving file: {}", e); },
                        Ok(()) => { app.document.set_saved_file(filename.clone()); },
                        _ => {},
                    }
                },
            }
        }

        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('O' as _, false) {
            mainmenu::load(app);
        }


        if !igIsAnyItemActive() {
            if igIsKeyPressed('A' as _, false) {
                app.document.inf_view.action = Action::Normal(NormalState::Default);
            }

            if igIsKeyPressed(' ' as _, false) {
                if let Some(DispatchView::Manual(m)) 
                     | Some(DispatchView::Auto(AutoDispatchView { dispatch: Some(m), .. })) 
                         = &mut app.document.dispatch_view {
                    m.play = !m.play;
                }
            }

            if igIsKeyPressed('D' as _, false) {
                app.document.inf_view.action = Action::DrawingLine(None);
            }

            if igIsKeyPressed('S' as _, false) {
                app.document.inf_view.action = Action::SelectObjectType;
            }
        }
    }
}
