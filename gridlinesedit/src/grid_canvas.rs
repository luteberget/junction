use imgui_sys_bindgen::sys::*;
use const_cstr::const_cstr;
use generational_arena::*;
use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Pt {
    x :i32,
    y :i32,
}

// not a linear length/coordinates system, each 
// edge has a physical length which is not related
// to the dx on screen. 
pub struct Railway {
    locations: Arena<Location>, 
    tracks: Arena<Track>,
}

pub struct SchematicCanvas {
    railway: Railway,
    selection: HashSet<Index>, // indexing locations. Select Track = select both a and b locations.
    default_grid_resolution: f64,
}

pub struct Track {
    end_a: Index,
    end_b: Index,
    length: f64,
}

pub enum Dir { Up, Down }
pub enum Side { Left, Right }


pub struct Location {
    node: Option<Node>, // AUTO or Node specified
}

pub enum Node {
    End,
    Continue,
    Switch(Dir,Side),
    Crossing,
}

pub struct GridCanvas {
    lines: Vec<(Pt,Pt)>,
    adding_line: Option<Pt>,
    scale: usize,
    translate :ImVec2,
}

impl GridCanvas {
    pub fn new() -> Self {
        GridCanvas {
            lines: Vec::new(),
            adding_line: None,
            scale: 35, // number of pixels per grid point, in interval [4, 100]
            translate: ImVec2 { x: 0.0, y: 0.0 },
        }
    }

    /// Converts and rounds a screen coordinate to the nearest point on the integer grid
    pub fn screen_to_world(&self, pt :ImVec2) -> Pt {
        let x = (self.translate.x + pt.x) / self.scale as f32;
        let y = (self.translate.y + pt.y) / self.scale as f32;
        Pt { x: x.round() as _ , y: y.round() as _ }
    }

    /// Convert a point on the integer grid into screen coordinates
    pub fn world_to_screen(&self, pt :Pt) -> ImVec2 {
        let x = ((self.scale as i32 * pt.x) as f32) - self.translate.x;
        let y = ((self.scale as i32 * pt.y) as f32) - self.translate.y;

        ImVec2 { x, y }
    }

    /// Return the rect of grid points within the current view.
    pub fn points_in_view(&self, size :ImVec2) -> (Pt,Pt) {
        let lo = self.screen_to_world(ImVec2 { x: 0.0, y: 0.0 });
        let hi = self.screen_to_world(size);
        (lo,hi)
    }

    pub fn route_line(from :Pt, to :Pt) -> Vec<(Pt,Pt)> {
        // diag
        let mut vec = Vec::new();
        let (dx,dy) = (to.x - from.x, to.y - from.y);
        let mut other = from;
        if dy.abs() > 0 {
            other = Pt { x: from.x + dy.abs() * dx.signum(), 
                         y: from.y + dy };
            vec.push((from, other));
        }
        if dx.abs() > 0 {
            let other_dx = to.x - other.x;
            let goal = Pt { x: other.x + if other_dx.signum() == dx.signum() { other_dx } else { 0 },
                            y: other.y };
            vec.push((other, goal));
        }
        vec
    }
}

pub fn grid_canvas(size: &ImVec2, canvas: &mut GridCanvas) {
    unsafe {
        let io = igGetIO();
        let draw_list = igGetWindowDrawList();
        let pos = igGetCursorScreenPos_nonUDT2();
        let pos = ImVec2 { x: pos.x, y: pos.y };

        let c1 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 } );
        let c2 = igGetColorU32Vec4(ImVec4 { x: 0.2, y: 0.5, z: 0.95, w: 1.0 } );
        let c3 = igGetColorU32Vec4(ImVec4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 } );
        let c4 = igGetColorU32Vec4(ImVec4 { x: 0.8, y: 0.8, z: 0.8, w: 1.0 } );

        ImDrawList_AddRectFilled(draw_list,
                        pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y },
                        c1, 0.0, 0);
        igInvisibleButton(const_cstr!("grid_canvas").as_ptr(), *size);
        ImDrawList_PushClipRect(draw_list, pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y}, true);

        let pointer = (*io).MousePos;
        let pointer_incanvas = ImVec2 { x: pointer.x - pos.x, y: pointer.y - pos.y };
        let pointer_grid = canvas.screen_to_world(pointer_incanvas);

        let line = |c :ImU32,p1 :&ImVec2,p2 :&ImVec2| {
			ImDrawList_AddLine(draw_list,
				   ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y },
				   ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
				   c, 2.0);
        };

        // Drawing or adding line
        match (igIsItemHovered(0), igIsMouseDown(0), &mut canvas.adding_line) {
            (true, true, None) => {
                canvas.adding_line = Some(pointer_grid);
            },
            (_, false, Some(pt)) => {
                for l in GridCanvas::route_line(*pt, pointer_grid) {
                    canvas.lines.push(l);
                }
                canvas.adding_line = None;
            },
            _ => {},
        };

        // Draw permanent lines
        for (p1,p2) in &canvas.lines {
            line(c2, &canvas.world_to_screen(*p1), &canvas.world_to_screen(*p2));
        }

        // Draw temporary line
        if let Some(pt) = &canvas.adding_line {
            for (p1,p2) in GridCanvas::route_line(*pt, pointer_grid) {
                line(c3, &canvas.world_to_screen(p1), &canvas.world_to_screen(p2));
            }
        }

        // Draw grid + highlight on closest point if hovering?
        let (lo,hi) = canvas.points_in_view(*size);
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                let pt = canvas.world_to_screen(Pt { x, y });
                ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + pt.x, y: pos.y + pt.y },
                                           3.0, c4, 4);
            }
        }

        ImDrawList_PopClipRect(draw_list);
    }
}

