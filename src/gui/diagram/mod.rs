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
            let viewport = dv.viewport.as_ref().unwrap();

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

fn diagram_toolbar(dv :&mut ManualDispatchView) {
    unsafe {
    let label = if dv.play { const_cstr!("pause") }
                else { const_cstr!("play") };
    if igButton(label.as_ptr(), ImVec2::zero()) {
        dv.play = !dv.play;
    }
    }
}
