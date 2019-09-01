use const_cstr::*;
use backend_glfw::imgui::*;

use crate::app::*;
use crate::document::dispatch::*;
use crate::gui::widgets;
use crate::config::*;
use crate::app::*;
use crate::document::*;
mod draw;

pub fn diagram_view(app :&mut App, dv :&mut ManualDispatchView) {
    unsafe {

    // Diagram accesses the following data:
    //  - app.document.dispatch_view 
    //    - either manual: dispatch idx, time, play
    //    - or auto:  optional dispatch idx, time, play
    //  - and app.document.data().dispatch
    //        ... or app.document.data().plandispatchdispatch
    //    ... to acccess simulation history and prepared visual 
    //        elements (tvd boxes and such).
    //
    // And then modifies the app by:
    //  - 

    diagram_toolbar(dv);

    let size = igGetContentRegionAvail_nonUDT2().into();
    widgets::canvas(size,
                app.config.color_u32(RailUIColorName::CanvasBackground),
                const_cstr!("diag").as_ptr(),
                app.document.inf_view.view.clone(),
                |draw| {

        draw::diagram(app, dv, draw);
        draw::command_icons(app, dv, draw);
        draw::time_slider(app, dv, draw);
        Some(())
    });
}
}

fn diagram_toolbar(dv :&mut ManualDispatchView) {
    unsafe {
    let label = if dv.play { const_cstr!("pause") }
                else { const_cstr!("play") };
    if igButton(label.as_ptr(), ImVec2::zero()) {
        dv.play = !dv.play;
    }
    }
}
