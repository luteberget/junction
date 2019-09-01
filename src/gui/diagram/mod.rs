use nalgebra_glm as glm;
use const_cstr::*;
use backend_glfw::imgui::*;

use crate::app::*;
use crate::document::dispatch::*;
use crate::document::analysis::*;
use crate::gui::widgets;
use crate::config::*;
use crate::app::*;
use crate::document::*;
use crate::gui::widgets::Draw;

mod draw;

pub fn diagram_view(config :&Config, analysis :&Analysis, dv :&mut ManualDispatchView, graph :&DispatchOutput) {
    unsafe {
        diagram_toolbar(dv);
        let size = igGetContentRegionAvail_nonUDT2().into();
        widgets::canvas(size,
                    config.color_u32(RailUIColorName::GraphBackground),
                    const_cstr!("diag").as_ptr(),
                    |draw| {

            if dv.viewport.is_none() { dv.viewport = Some(DiagramViewport {
                    time: (graph.time_interval.0 as _, graph.time_interval.1 as _),
                    pos: (graph.pos_interval.0 as _, graph.pos_interval.1 as _),
                }); }
            let viewport = dv.viewport.as_mut().unwrap();

            scroll(draw, viewport);

            // Need to get a DispatchOutput from analysis.
            draw::diagram(config, graph, draw, viewport);
            draw::command_icons(config, analysis, graph, viewport, draw);
            draw::time_slider(config, draw, viewport, dv.time);

            let mouse_time = glm::lerp_scalar(viewport.time.0 as f32, viewport.time.1 as f32,
                                              draw.mouse.y/draw.size.y);

            if igIsItemHovered(0) && igIsMouseDown(0) {
                dv.time = mouse_time as f64;
            }

            Some(())
        });
    }
}

fn scroll(draw :&Draw, viewport :&mut DiagramViewport) {
    fn translate((a,b) :(f64,f64), d:f64) -> (f64,f64) { (a+d,b+d) }
    fn dilate((a,b) :(f64,f64), f :f64) -> (f64,f64) {
        let mid = 0.5*(a+b);
        let dist = 0.5*(b-a);
        (mid - f*dist, mid + f*dist)
    }

    unsafe {
        if !igIsItemHovered(0) { return; }
        let io = igGetIO();
        let wheel = (*io).MouseWheel;
        if wheel != 0.0 {
            let factor = 1.0 + (-wheel as f64 / 20.0);
            viewport.time = dilate(viewport.time, factor);
            viewport.pos = dilate(viewport.pos, factor);
        }

        if ((*io).KeyCtrl && igIsMouseDragging(0,-1.0)) || igIsMouseDragging(2, -1.0) {
            let mouse_delta = (*io).MouseDelta;
            let delta = ImVec2 { x: -mouse_delta.x / draw.size.x * 
               ( (viewport.pos.1 - viewport.pos.0) as f32) ,
                                 y: -mouse_delta.y / draw.size.y * 
                                     ((viewport.time.1 - viewport.time.0) as f32), };

            viewport.pos = translate(viewport.pos, delta.x as _);
            viewport.time = translate(viewport.time, delta.y as _);
        }
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
