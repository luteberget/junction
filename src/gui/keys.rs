use crate::app::App;
use crate::document::objects::*;
use crate::document::infview::*;
use crate::gui::mainmenu;

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

        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('S' as _, false) {
            //match file::save(filename, app.document.model().clone()) {
            //    Err(e) => { error!("Error saving file: {}", e); },
            //    Ok(()) => { app.document.fileinfo.set_saved(); },
            //};
            // TODO
        }

        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('S' as _, false) {
            //mainmenu::save_as(doc);
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('O' as _, false) {
            //mainmenu::load(doc, self, diagram);
        }


        // TODO play
//        if !(*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed(' ' as _, false) {
//            if let Some((_,_,play)) = self.active_dispatch.as_mut() {
//                *play = !*play;
//            }
//        }


        if igIsKeyPressed('A' as _, false) {
            app.document.inf_view.action = Action::Normal(NormalState::Default);
        }

        if igIsKeyPressed('D' as _, false) {
            app.document.inf_view.action = Action::DrawingLine(None);
        }

        if igIsKeyPressed('S' as _, false) {
            let current_object_function = if let Action::InsertObject(Some(obj)) = 
                            &app.document.inf_view.action {
                obj.functions.iter().next()
            } else { None };

            if current_object_function == Some(&Function::Detector) {
                    app.document.inf_view.action = Action::InsertObject(Some(
                            Object {
                                loc: glm::vec2(0.0,0.0),
                                tangent :glm::vec2(1,0),
                                functions: vec![Function::MainSignal { has_distant: false }] } ));

            } else {
                    app.document.inf_view.action = Action::InsertObject(Some(
                            Object {
                                loc: glm::vec2(0.0,0.0),
                                tangent :glm::vec2(1,0),
                                functions: vec![Function::Detector] } ));
            }
        }
    }
}
