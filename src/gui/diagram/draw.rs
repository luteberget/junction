use const_cstr::*;
use matches::*;
use backend_glfw::imgui::*;

use crate::config::*;
use crate::app::*;
use crate::gui::widgets;
use crate::gui::widgets::Draw;
use crate::document::dispatch::*;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::document::*;
use crate::gui::diagram::DiagramViewAction;

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


            let (mut p1, mut p2) = (Polyline::new(), Polyline::new());
            p1.add_bezier_interpolated(
                             to_screen(draw, view, s.start_time + 0.0/3.0*s.dt, s.kms[0]),
                             to_screen(draw, view, s.start_time + 1.0/3.0*s.dt, s.kms[1]),
                             to_screen(draw, view, s.start_time + 2.0/3.0*s.dt, s.kms[2]),
                             to_screen(draw, view, s.start_time + 3.0/3.0*s.dt, s.kms[3])
                             );
            p2.add_bezier_interpolated(
                             to_screen(draw, view, s.start_time + 0.0/3.0*s.dt, s.end_kms[0]),
                             to_screen(draw, view, s.start_time + 1.0/3.0*s.dt, s.end_kms[1]),
                             to_screen(draw, view, s.start_time + 2.0/3.0*s.dt, s.end_kms[2]),
                             to_screen(draw, view, s.start_time + 3.0/3.0*s.dt, s.end_kms[3]),
                             );

            //Polyline::draw_triangulate_monotone_y(&p1,&p2,draw,col_train_rear);
            p1.draw_path(draw, col_train_rear);
            p2.draw_path(draw, col_train_rear);
        }
    }
}

struct Polyline {
    pub path :Vec<ImVec2>,
}

impl Polyline {
    pub fn draw_path(&self, draw :&Draw, col :u32) {
        unsafe {
            ImDrawList_AddPolyline(draw.draw_list, self.path.as_ptr(), self.path.len() as _, col, false, 2.0);
        }
    }
    pub fn draw_triangulate_monotone_y(p1 :&Polyline, p2 :&Polyline, draw :&Draw, col :u32) {
        if p1.path.len() <= 1 || p2.path.len() <= 1 { return; }
        let (mut i,mut j) = (0,0);
        while i+1 < p1.path.len() || j+1 < p2.path.len() {
            // advance one of the pointers
            let advance_p1 = if !(i+1 < p1.path.len()) { 
                false
            } else if !(j+1 < p2.path.len()) {
                true
            } else { p1.path[i+1].y < p2.path[j+1].y };

            if advance_p1 {
                unsafe { ImDrawList_AddTriangleFilled(draw.draw_list, p1.path[i], p1.path[i+1], p2.path[j], col); }
                i += 1;
            } else {
                unsafe { ImDrawList_AddTriangleFilled(draw.draw_list, p2.path[j], p2.path[j+1], p1.path[i], col); }
                j += 1;
            }
        }
    }
    pub fn new() -> Polyline { Polyline { path: Vec::with_capacity(8) } }
    pub fn add_bezier_interpolated(&mut self, p1 :ImVec2, y2 :ImVec2, y3 :ImVec2, p4 :ImVec2) {
        let tess_tol = 1.25;
        let p2 = (-5.0*p1 + 18.0*y2 - 9.0*y3 + 2.0*p4) / 6.0;
        let p3 = (-5.0*p4 + 18.0*y3 - 9.0*y2 + 2.0*p1) / 6.0;
        self.path.push(p1);
        self.path_bezier_to_casteljau(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, p4.x, p4.y, tess_tol, 0);
    }
    pub fn path_bezier_to_casteljau(&mut self, 
                                    x1 :f32, y1 :f32, x2 :f32, y2 :f32, 
                                    x3 :f32, y3 :f32, x4 :f32, y4 :f32, tess_tol :f32, level :i32) {
        let dx = x4 - x1;
        let dy = y4 - y1;
        let mut d2 = ((x2 - x4) * dy - (y2 - y4) * dx);
        let mut d3 = ((x3 - x4) * dy - (y3 - y4) * dx);
        d2 = if d2 >= 0.0 { d2 } else { -d2 };
        d3 = if d3 >= 0.0 { d3 } else { -d3 };
        if ((d2+d3) * (d2+d3) < tess_tol * (dx*dx + dy*dy))
        {
            self.path.push(ImVec2 { x: x4, y: y4 });
        }
        else if (level < 10)
        {
            let x12 = (x1+x2)*0.5;       let y12 = (y1+y2)*0.5;
            let x23 = (x2+x3)*0.5;       let y23 = (y2+y3)*0.5;
            let x34 = (x3+x4)*0.5;       let y34 = (y3+y4)*0.5;
            let x123 = (x12+x23)*0.5;    let y123 = (y12+y23)*0.5;
            let x234 = (x23+x34)*0.5;    let y234 = (y23+y34)*0.5;
            let x1234 = (x123+x234)*0.5; let y1234 = (y123+y234)*0.5;

            self.path_bezier_to_casteljau(x1,y1,        x12,y12,    x123,y123,  x1234,y1234, tess_tol, level+1);
            self.path_bezier_to_casteljau(x1234,y1234,  x234,y234,  x34,y34,    x4,y4,       tess_tol, level+1);
        }
    }
}

pub fn command_icons(config :&Config, analysis :&Analysis, 
                     graphics :&DispatchOutput,
                     draw :&Draw, 
                     dv :&mut ManualDispatchView) -> Option<DiagramViewAction> {

    let mut action = None;

    let col1 = config.color_u32(RailUIColorName::GraphCommand);
    let col2 = config.color_u32(RailUIColorName::GraphCommandBorder);
    let il = analysis.data().interlocking.as_ref()?;
    let dgraph = analysis.data().dgraph.as_ref()?;
    let dispatch = &graphics.dispatch;

    for (cmd_idx,(cmd_id,(cmd_t,cmd))) in dispatch.commands.iter().enumerate() {
        let km = match cmd { Command::Route(routespec) | Command::Train(_,routespec) => {
            il.find_route(routespec).and_then(|r_id| il.routes[*r_id].start_mileage(dgraph))
        }}.unwrap_or(0.0);
        unsafe {
            let half_icon_size = ImVec2 { x: 4.0, y: 4.0 };
            let p = to_screen(draw, dv.viewport.as_ref().unwrap(), *cmd_t, km);
            ImDrawList_AddRectFilled(draw.draw_list, 
                                     p - half_icon_size, 
                                     p + half_icon_size, col1, 0.0, 0);
            ImDrawList_AddRect(draw.draw_list, 
                               p - half_icon_size, 
                               p + half_icon_size, col2, 0.0, 0, 1.0);

            if igIsItemHovered(0) && (p-draw.pos-draw.mouse).length_sq() < 5.0*5.0 {
                igBeginTooltip();
                widgets::show_text("command");
                igEndTooltip();

                if igIsMouseDown(0) && matches!(dv.action, ManualDispatchViewAction::None) {
                    dv.action = ManualDispatchViewAction::DragCommandTime { idx: cmd_idx, id :*cmd_id };
                }

                if igIsMouseClicked(1, false) {
                    dv.selected_command = Some(*cmd_id);
                    igOpenPopup(const_cstr!("cmded").as_ptr());
                }
            }
        }
    }
    action
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

