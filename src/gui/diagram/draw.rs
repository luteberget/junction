use backend_glfw::imgui::*;

use crate::config::*;
use crate::app::*;
use crate::gui::widgets::*;
use crate::document::dispatch::*;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::document::*;

pub fn diagram(config :&Config, graphics :&DispatchOutput, draw :&Draw, view :&DiagramViewport) {
    let col_res = config.color_u32(RailUIColorName::GraphBlockReserved);
    let col_box = config.color_u32(RailUIColorName::GraphBlockBorder);
    let col_occ = config.color_u32(RailUIColorName::GraphBlockOccupied);

    let col_train_front = config.color_u32(RailUIColorName::GraphTrainFront);
    let col_train_rear = config.color_u32(RailUIColorName::GraphTrainRear);

    unsafe {
        for block in &graphics.diagram.blocks {
            if block.reserved.0 < block.occupied.0 {
                ImDrawList_AddRectFilled(draw.draw_list,
                     to_screen(draw, view, block.reserved.0, block.pos.0),
                     to_screen(draw, view, block.occupied.0, block.pos.1),
                     col_res, 0.0, 0);
            }
            // Occupied
            ImDrawList_AddRectFilled(draw.draw_list,
                     to_screen(draw, view, block.occupied.0, block.pos.0),
                     to_screen(draw, view, block.occupied.1, block.pos.1),
                     col_occ, 0.0, 0);

            // Reserved after
            if block.reserved.1 > block.occupied.1 {
                ImDrawList_AddRectFilled(draw.draw_list,
                     to_screen(draw, view, block.occupied.1, block.pos.0),
                     to_screen(draw, view, block.reserved.1, block.pos.1),
                     col_res, 0.0, 0);

            }

            ImDrawList_AddRect(draw.draw_list,
                to_screen(draw, view, block.reserved.0, block.pos.0),
                to_screen(draw, view, block.reserved.1, block.pos.1),
                col_box, 0.0, 0, 1.0);
        }
    }

    for graph in &graphics.diagram.trains {
        for s in &graph.segments {
            draw_interpolate(draw.draw_list,
                             to_screen(draw, view, s.start_time + 0.0/3.0*s.dt, s.kms[0]),
                             to_screen(draw, view, s.start_time + 1.0/3.0*s.dt, s.kms[1]),
                             to_screen(draw, view, s.start_time + 2.0/3.0*s.dt, s.kms[2]),
                             to_screen(draw, view, s.start_time + 3.0/3.0*s.dt, s.kms[3]),
                             col_train_front);
            draw_interpolate(draw.draw_list,
                             to_screen(draw, view, s.start_time + 0.0/3.0*s.dt, s.end_kms[0]),
                             to_screen(draw, view, s.start_time + 1.0/3.0*s.dt, s.end_kms[1]),
                             to_screen(draw, view, s.start_time + 2.0/3.0*s.dt, s.end_kms[2]),
                             to_screen(draw, view, s.start_time + 3.0/3.0*s.dt, s.end_kms[3]),
                             col_train_rear);
        }
    }
}

pub fn command_icons(config :&Config, analysis :&Analysis, 
                     graphics :&DispatchOutput,
                     view :&DiagramViewport, draw :&Draw) -> Option<()> {

    let col1 = config.color_u32(RailUIColorName::GraphCommand);
    let col2 = config.color_u32(RailUIColorName::GraphCommandBorder);
    let il = analysis.data().interlocking.as_ref()?;
    let dgraph = analysis.data().dgraph.as_ref()?;
    let dispatch = &graphics.dispatch;

    for (idx,(t,cmd)) in dispatch.0.iter().enumerate() {
        let node = match cmd { Command::Route(route) | Command::Train(_,route) => {
            route.from }};

        //let km = match node {
        //    Ref::Node(pt) => { },
        //}
        //node.and_then(|n| dgraph.mileage.get(n).cloned()).unwrap_or(0.0);

        let km = 0.0;
        unsafe {
            let half_icon_size = ImVec2 { x: 4.0, y: 4.0 };
            let p = to_screen(draw, view, *t, km);
            ImDrawList_AddRectFilled(draw.draw_list, 
                                     p - half_icon_size, 
                                     p + half_icon_size, col1, 0.0, 0);
            ImDrawList_AddRect(draw.draw_list, 
                               p - half_icon_size, 
                               p + half_icon_size, col1, 0.0, 0, 4.0);
        }
    }

    Some(())
}

pub fn time_slider(config :&Config, draw :&Draw, viewport :&DiagramViewport, t :f64) {
	unsafe {
		let c1 = config.color_u32(RailUIColorName::GraphTimeSlider);
		let c2 = config.color_u32(RailUIColorName::GraphTimeSliderText);

		// Draw the line
		ImDrawList_AddLine(draw.draw_list,
                           to_screen(draw, viewport, t, viewport.pos.0),
                           to_screen(draw, viewport, t, viewport.pos.1),
						   c1, 2.0);

		let text = format!("t = {:.3}", t);
		ImDrawList_AddText(draw.draw_list,
                           to_screen(draw, viewport, t, viewport.pos.0),
						   c2,
						   text.as_ptr() as _ , text.as_ptr().offset(text.len() as isize) as _ );
	}
}

pub fn to_screen(draw :&Draw, v :&DiagramViewport, t: f64, x :f64) -> ImVec2 {
    ImVec2 {
        x: draw.pos.x + draw.size.x*(((x - v.pos.0)/(v.pos.1 - v.pos.0)) as f32),
        y: draw.pos.y + draw.size.y*(((t - v.time.0)/(v.time.1 - v.time.0)) as f32),
    }
}

pub fn draw_interpolate(draw_list :*mut ImDrawList, p0 :ImVec2, y1 :ImVec2, y2 :ImVec2, p3 :ImVec2, col
:u32) {
    // https://web.archive.org/web/20131225210855/http://people.sc.fsu.edu/~jburkardt/html/bezier_inter polation.html
    let p1 = (-5.0*p0 + 18.0*y1 - 9.0*y2 + 2.0*p3) / 6.0;
    let p2 = (-5.0*p3 + 18.0*y2 - 9.0*y1 + 2.0*p0) / 6.0;
    unsafe {
    ImDrawList_AddBezierCurve(draw_list, p0,p1,p2,p3, col, 2.0, 0);
    }
}

