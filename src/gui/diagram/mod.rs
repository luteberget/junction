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

#[derive(Copy,Clone)]
pub enum DiagramViewAction {
    DeleteCommand { id :usize },
    MoveCommand { idx :usize, id :usize, t :f64 },
}

pub fn default_viewport(graph :&DispatchOutput) -> DiagramViewport {
    DiagramViewport {
        time: (graph.time_interval.0 as _, graph.time_interval.1 as _),
        pos: (graph.pos_interval.0 as _, graph.pos_interval.1 as _),
    }
}

pub fn diagram_view(config :&Config, analysis :&Analysis, dv :&mut ManualDispatchView, graph :&DispatchOutput) -> Option<DiagramViewAction> {
    let mut action = None;
    unsafe {
        igSameLine(0.0,-1.0);
        diagram_toolbar(dv, graph);
        let size = igGetContentRegionAvail_nonUDT2().into();
        widgets::canvas(size,
                    config.color_u32(RailUIColorName::GraphBackground),
                    const_cstr!("diag").as_ptr(),
                    |draw| {

            if dv.viewport.is_none() { dv.viewport = Some(default_viewport(graph)); }

            // Need to get a DispatchOutput from analysis.
            draw::diagram(config, graph, draw, dv.viewport.as_ref().unwrap());
            action = draw::command_icons(config, analysis, graph, draw, dv).or(action);
            draw::time_slider(config, draw, dv.viewport.as_ref().unwrap(), dv.time);

            let viewport = dv.viewport.as_mut().unwrap();
            scroll(draw, viewport);

            let mouse_time = glm::lerp_scalar(viewport.time.0 as f32, viewport.time.1 as f32,
                                              draw.mouse.y/draw.size.y);

            match dv.action {
                ManualDispatchViewAction::None => {},
                ManualDispatchViewAction::DragCommandTime { idx, id } => {
                    action = Some(DiagramViewAction::MoveCommand { idx, id, t: mouse_time as f64 });
                    if !igIsMouseDown(0) {
                        dv.action = ManualDispatchViewAction::None;
                    }
                },
            }

            if igBeginPopup(const_cstr!("cmded").as_ptr(), 0 as _) {
                if let Some(selection) = dv.selected_command {
                    if igSelectable(const_cstr!("Delete").as_ptr(), false, 0 as _, ImVec2::zero()) {
                        action = Some(DiagramViewAction::DeleteCommand { id: selection });
                    }
                }
                igEndPopup();
            }

            if igIsItemHovered(0) && igIsMouseDown(0) {
                dv.time = mouse_time as f64;
            }

            Some(())
        });
    }
    action
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


fn diagram_toolbar(dv :&mut ManualDispatchView, graph :&DispatchOutput) {
    unsafe {
    let label = if dv.play { const_cstr!("\u{f04c}") }
                else { const_cstr!("\u{f04b}") };
    if igButton(label.as_ptr(), ImVec2::zero()) {
        dv.play = !dv.play;
    }
    igSameLine(0.0,-1.0);
    if igButton(const_cstr!("\u{f0b2}").as_ptr(), ImVec2::zero()) {
        dv.viewport = Some(default_viewport(graph));
    }
    }
}
