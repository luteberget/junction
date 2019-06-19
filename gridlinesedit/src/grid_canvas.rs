use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::text::*;
use const_cstr::const_cstr;
use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};
use matches::matches;


use crate::pt::*;
use crate::symset::*;
use crate::topology::*;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Document {
    pieces :SymSet<Pt>,
    // TODO symbols, node types, etc.
    // TODO how to do naming etc in dispatches / movements.
    railway :Option<Railway>,
    objects :Vec<(usize,f32,Object)>,
    node_data :HashMap<Pt, NDType>, // copied into railway when topology has changed
}

#[derive(Debug)]
pub struct SchematicCanvas {
    document :Document,
    tool: Tool,
    selected_pieces :HashSet<(Pt,Pt)>,
    scale: Option<usize>,
    translate :Option<ImVec2>,
    adding_line :Option<Pt>,
    adding_object: Option<((f32,f32),(Pt,Pt))>,  // Pt-continuous
    editing_node: Option<Pt>,
}


pub fn lsqr(a :(f32,f32), b :(f32,f32)) -> f32 {
    let dx = b.0-a.0;
    let dy = b.1-a.1;
    dx*dx+dy*dy
}

pub fn dot(a :(f32,f32), b :(f32,f32)) -> f32 {
    a.0*b.0 + a.1*b.1
}

// TODO use a library

pub fn project_to_line((x,y) :(f32,f32), (p1,p2) :(Pt,Pt)) -> (f32,f32) {
    let p1 = (p1.x as f32, p1.y as f32);
    let p2 = (p2.x as f32, p2.y as f32);
    let l2 = lsqr(p1,p2);
    let t = (dot( (x - p1.0, y - p1.1), (p2.0 - p1.0, p2.1 - p1.1 )) / l2 ). min(1.0).max(0.0);
    let proj = ( p1.0 + t * (p2.0-p1.0),
                 p1.1 + t * (p2.1-p1.1));
    proj
}

pub fn dist_to_line_sqr((x,y) :(f32,f32), (p1,p2) :(Pt, Pt)) -> f32 {
    let proj = project_to_line((x,y),(p1,p2));
    lsqr((x,y), proj)
}

#[derive(Debug)]
pub enum Tool { Scroll, Draw, Modify }

impl SchematicCanvas {


    pub fn refresh(&mut self) {
        self.document.railway = to_railway(self.document.pieces.clone(), 
                                           &self.document.node_data,
                                           50.0).ok();
    }

    pub fn new() -> Self {

        SchematicCanvas {
            document: Document {
                pieces: SymSet::new(),
                objects: Vec::new(),
                node_data: HashMap::new(),
                railway: None,
            },
            selected_pieces: HashSet::new(),
            tool: Tool::Scroll,
            adding_line: None,
            scale: None,
            translate :None,//ImVec2{ x:0.0, y:0.0 },
            adding_object: None,
            editing_node: None,
        }
    }

    pub fn closest_edge(&self, (x,y) :(f32,f32)) -> Option<(Pt,Pt)> {
        let (xl,xh) = ((x-0.0) as i32, ((x+x.signum()*1.0) as i32));
        let (yl,yh) = ((y-0.0) as i32, ((y+y.signum()*1.0) as i32));
        let candidates = [
            ((xl,yl),(xh,yl)),
            ((xl,yh),(xh,yh)),
            ((xl,yl),(xl,yh)),
            ((xh,yl),(xh,yh)),
            ((xl,yl),(xh,yh)),
            ((xl,yh),(xh,yl)) ];
        let (mut d, mut l) = (5.0, None);
        for (p1,p2) in candidates.iter() {
            let p1 = Pt { x: p1.0, y: p1.1};
            let p2 = Pt { x: p2.0, y: p2.1};
            println!("TRying {:?}->{:?}", p1,p2);
            if !self.document.pieces.contains((p1,p2)) { continue; }
            let d2 = dist_to_line_sqr((x,y),(p1,p2));
            println!("dist {:?}", d2);
            if d2 < d {
                d = d2;
                l = Some((p1,p2))
            }
        }
        l
    }

    pub fn screen_to_world_cont(&self, pt :ImVec2) -> (f32,f32) {
        let t = self.translate.unwrap_or(ImVec2{x:0.0,y:0.0});
        let s = self.scale.unwrap_or(35);
        let x =  (t.x + pt.x) / s as f32;
        let y = -(t.y + pt.y) / s as f32;
        (x,y)
    }

    /// Converts and rounds a screen coordinate to the nearest point on the integer grid
    pub fn screen_to_world(&self, pt :ImVec2) -> Pt {
        let (x,y) = self.screen_to_world_cont(pt);
        Pt { x: x.round() as _ , y: y.round() as _ }
    }


    pub fn world_to_screen_cont(&self, pt :(f32,f32)) -> ImVec2 {
        let t = self.translate.unwrap_or(ImVec2{x:0.0,y:0.0});
        let s = self.scale.unwrap_or(35);
        let x = ((s as f32 * pt.0) as f32)  - t.x;
        let y = ((s as f32 * -pt.1) as f32) - t.y;

        ImVec2 { x, y }
    }

    /// Convert a point on the integer grid into screen coordinates
    pub fn world_to_screen(&self, pt :Pt) -> ImVec2 {
        let t = self.translate.unwrap_or(ImVec2{x:0.0,y:0.0});
        let s = self.scale.unwrap_or(35);
        let x = ((s as i32 * pt.x) as f32)  - t.x;
        let y = ((s as i32 * -pt.y) as f32) - t.y;

        ImVec2 { x, y }
    }

    /// Return the rect of grid points within the current view.
    pub fn points_in_view(&self, size :ImVec2) -> (Pt,Pt) {
        let lo = self.screen_to_world(ImVec2 { x: 0.0, y: size.y });
        let hi = self.screen_to_world(ImVec2 { x: size.x, y: 0.0 });
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
            if other != goal {
                vec.push((other, goal));
            }
        }
        vec
    }
}

pub fn unit_step_diag_line(p1 :Pt, p2 :Pt) -> impl Iterator<Item = Pt> {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (0..=(dx.abs().max(dy.abs()))).map(move |d| Pt { x: p1.x + d * dx.signum(),
                                                     y: p1.y + d * dy.signum() } )
}

pub fn schematic_hotkeys(c :&mut SchematicCanvas) {
    use std::fs::File;
    use std::path::Path;
    let json_file_path = Path::new("doc.json");

    unsafe {
        let io = igGetIO();
        if (*io).KeyCtrl && igIsKeyPressed('S' as _, false) {
            let json_file = File::create(json_file_path).expect("could not create file");
            serde_json::to_writer(json_file, &c.document).expect("could not write to file");
            println!("Saved.{}",serde_json::to_string(&c.document).expect("could not serialize"));
        }
        if (*io).KeyCtrl && igIsKeyPressed('L' as _, false) {
            let json_file = File::open(json_file_path).expect("file not found");
            c.document = serde_json::from_reader(json_file).expect("error while reading json");
            println!("Loaded.\n{:?}",c);
        }
    }
}


pub fn tool_button(name :*const i8, selected: bool) -> bool {
    unsafe {
        if selected {

            let c1 = ImVec4 { x: 0.4, y: 0.65,  z: 0.4, w: 1.0 };
            let c2 = ImVec4 { x: 0.5, y: 0.85, z: 0.5, w: 1.0 };
            let c3 = ImVec4 { x: 0.6, y: 0.9,  z: 0.6, w: 1.0 };
            igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        } 
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        if selected {
            igPopStyleColor(3);
        } 
        clicked
    }
}

pub fn schematic_canvas(size: &ImVec2, model: &mut SchematicCanvas) {
    unsafe {
        schematic_hotkeys(model);

        if tool_button(const_cstr!("Mod").as_ptr(), matches!(&model.tool, Tool::Modify)) {
            model.tool = Tool::Modify;
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("Scr").as_ptr(), matches!(&model.tool, Tool::Scroll)) {
            model.tool = Tool::Scroll;
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("Drw").as_ptr(), matches!(&model.tool, Tool::Draw)) {
            model.tool = Tool::Draw;
        }


        let io = igGetIO();
        let draw_list = igGetWindowDrawList();
        let pos = igGetCursorScreenPos_nonUDT2();
        let pos = ImVec2 { x: pos.x, y: pos.y };

        let c1 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 } );
        let c2 = igGetColorU32Vec4(ImVec4 { x: 0.2, y: 0.5, z: 0.95, w: 1.0 } );
        let c3 = igGetColorU32Vec4(ImVec4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 } );
        let c4 = igGetColorU32Vec4(ImVec4 { x: 0.8, y: 0.8, z: 0.8, w: 1.0 } );
        let c5 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.4, z: 0.8, w: 1.0 } );
        let c6 = igGetColorU32Vec4(ImVec4 { x: 0.8, y: 0.8, z: 0.5, w: 0.25 } );

        ImDrawList_AddRectFilled(draw_list,
                        pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y },
                        c1, 0.0, 0);
        let clicked = igInvisibleButton(const_cstr!("grid_canvas").as_ptr(), *size);
        ImDrawList_PushClipRect(draw_list, pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y}, true);

        let special_key = (*io).KeyCtrl | (*io).KeyAlt | (*io).KeySuper;
        if (igIsItemActive() || !igIsAnyItemActive()) && !special_key {
            // Handle keys
            if igIsKeyPressed('A' as _, false) { 
                model.tool = Tool::Modify; 
                println!("tool {:?}", model.tool);
            }
            if igIsKeyPressed('S' as _, false) {
                model.tool = Tool::Scroll; 
                println!("tool {:?}", model.tool);
            }
            if igIsKeyPressed('D' as _, false) {
                model.tool = Tool::Draw; 
                println!("tool {:?}", model.tool);
            }
        }

        let pointer = (*io).MousePos;
        let pointer_incanvas = ImVec2 { x: pointer.x - pos.x, y: pointer.y - pos.y };
        let pointer_grid = model.screen_to_world(pointer_incanvas);

        let line = |c :ImU32,p1 :&ImVec2,p2 :&ImVec2| {
            ImDrawList_AddLine(draw_list,
                   ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y },
                   ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
                   c, 2.0);
        };


        if let Tool::Draw = &model.tool {
            // context menu at edge for adding objects
            if igIsItemHovered(0) && igIsMouseReleased(1) {
                // NOTE some duplicate state here, model contains the point clicked
                // and imgui popup state contains the location of the popup window?
                let loc = model.screen_to_world_cont(pointer_incanvas);
                if let Some(edge) = model.closest_edge(loc) {
                    println!("OBJ {:?}", (loc,edge));
                    model.adding_object = Some((loc,edge));
                }
                igOpenPopup(const_cstr!("ctx").as_ptr());
            } 
                

            // Drawing or adding line
            match (igIsItemHovered(0), igIsMouseDown(0), &mut model.adding_line) {
                (true, true, None)   => { model.adding_line = Some(pointer_grid); },
                (_, false, Some(pt)) => {
                    for (p1,p2) in SchematicCanvas::route_line(*pt, pointer_grid) {
                        for (p1,p2) in unit_step_diag_line(p1, p2).zip(
                                unit_step_diag_line(p1, p2).skip(1)) {
                            println!("ADdding {:?} {:?}", p1,p2);
                            model.document.pieces.insert((p1,p2));
                        }
                    }
                    model.refresh();
                    println!("Got new railway:");
                    println!("{:#?}", &model.document.railway);
                    model.adding_line = None;
                },
                _ => {},
            };
        }

        if let Tool::Scroll = &model.tool {
            if igIsMouseDown(0) {
                if model.translate.is_none() { model.translate = Some(ImVec2 { x: 0.0, y: 0.0}); }
                let t = model.translate.as_mut().unwrap();
                t.x -= (*io).MouseDelta.x;
                t.y -= (*io).MouseDelta.y;

            }
        }

        if let Tool::Modify = &model.tool {

            if igIsItemHovered(0) && igIsMouseReleased(1) {
                // right click opens context menu on node
                let loc = model.screen_to_world(pointer_incanvas);
                // find node
                if let Some(r) = &model.document.railway {
                    for (p,_,_) in &r.locations {
                        if *p == loc {
                            model.editing_node = Some(*p);
                            igOpenPopup(const_cstr!("ctx").as_ptr());
                        }
                    }
                }
            }


            // TODO just sketching
            //
            //
            // 1. start selection capture window
            // 2. select (single/add/remove) object
            // (  3. selection context menu   )
            // 4. move things by dragging already selected object.

            //let hov = igIsItemHovered(0);
            //let dn  = igIsMouseDown(0);

            //if clicked && let obj = near() {
            //    model.selection = std::iter::once(obj).collect();
            //} else if dragged {
            //    if near_any(model.selection) {
            //        move_objs(model.selection, (*io).MouseDelta);
            //    } else {
            //        start_selection_window();
            //    }
            //}
        }

        // Draw permanent lines
        for (p,set) in &model.document.pieces.map {
            for q in set {
                if p < q {
                    line(c2, &model.world_to_screen(*p), &model.world_to_screen(*q));
                }
            }
        }

        // Draw temporary line
        if let Some(pt) = &model.adding_line {
            for (p1,p2) in SchematicCanvas::route_line(*pt, pointer_grid) {
                line(c3, &model.world_to_screen(p1), &model.world_to_screen(p2));
            }
        }

        let gridc = if let Tool::Draw  = &model.tool {
            igGetColorU32Vec4(ImVec4 { x: 0.5, y: 0.5, z: 0.5, w: 0.90 } )
        }  else {
            igGetColorU32Vec4(ImVec4 { x: 0.5, y: 0.5, z: 0.5, w: 0.45 } )
        };

        // Draw grid + highlight on closest point if hovering?
        let (lo,hi) = model.points_in_view(*size);
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                let pt = model.world_to_screen(Pt { x, y });
                ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + pt.x, y: pos.y + pt.y },
                                           3.0, gridc, 4);
            }
        }

        // Draw nodes
        if let Some(r) = &model.document.railway {
            for (pt,typ,vc) in &r.locations {
                let p1 = model.world_to_screen(*pt);
                //let p2 = model.world_to_screen(Pt { x: pt.x + vc.x,
                //                                    y: pt.y + vc.y });
                match typ {
                    NDType::OpenEnd => {
                        //ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + p1.x, 
                            //y: pos.y + p1.y }, 6.0, c3, 8);
                        let tangent = vc;
                        let normal = Pt { x: -vc.y, y: vc.x };
                        let tl = ((tangent.x.abs() + tangent.y.abs()) as f32).sqrt();
                        let scale=5.0/tl;
                        let pt = ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y };
                        let pa = ImVec2 { x: pt.x + scale*(tangent.x as f32 + normal.x as f32),
                                          y: pt.y + scale*((-tangent.y) as f32 + (-normal.y  as f32)) };
                        let pb = ImVec2 { x: pt.x + scale*(tangent.x as f32 - normal.x as f32),
                                          y: pt.y + scale*((-tangent.y) as f32 + (-(-normal.y) as f32)) };
                        ImDrawList_AddLine(draw_list, pt,pa, c3, 2.0);
                        ImDrawList_AddLine(draw_list, pt,pb, c3, 2.0);

                    },
                    NDType::BufferStop => {
                        let tangent = vc;
                        let normal = Pt { x: -vc.y, y: vc.x };
                        let tl = ((tangent.x.abs() + tangent.y.abs()) as f32).sqrt();
                        let scale=5.0/tl;
                        let pt = ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y };
                        let pa = ImVec2 { x: pt.x + scale*( normal.x as f32),
                                          y: pt.y + scale*((-normal.y  as f32)) };
                        let pb = ImVec2 { x: pt.x + scale*(- normal.x as f32),
                                          y: pt.y + scale*((-(-normal.y) as f32)) };
                        ImDrawList_AddLine(draw_list, pt,pa, c4, 2.0);
                        ImDrawList_AddLine(draw_list, pt,pb, c4, 2.0);
                    },
                    NDType::Cont => {
                        ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + p1.x, 
                            y: pos.y + p1.y }, 6.0, c5, 8);
                        //let p2 = model.world_to_screen(Pt { x: pt.x + vc.x,
                        //                                    y: pt.y + vc.y });
                    },
                    NDType::Sw(side) => {
                        //let scale = model.scale.unwrap_or(35);
                        let scale = 15.0;
                        let p1 = ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y };
                        let p2 = ImVec2 { x: p1.x + scale*(vc.x as f32), y: p1.y + scale*(-vc.y as f32) };
                        let p3 = rotate(*vc, side.to_rotation());
                        let p3 = ImVec2 { x: p1.x + scale*(p3.x as f32), y: p1.y + scale*(-p3.y as f32) };
                       // ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + p1.x, 
                       //     y: pos.y + p1.y }, 6.0, c6, 4);
                        ImDrawList_AddTriangleFilled(draw_list, p1,p2,p3,
                                               c2);
                        
                    },
                    NDType::Err => {
                    },
                }
            }
        }

        // Highlight edge
        if let Some(((c1,c2),(p1,p2))) = &model.adding_object {
            let tangent = Pt { x: p2.x - p1.x, y: p2.y -p1.y };
            let normal = Pt { x: -tangent.y, y: tangent.x };

            let scale = 5.0;
            let p = project_to_line((*c1,*c2),(*p1,*p2)); // world coords
            let p = model.world_to_screen_cont(p); 
            let pa = ImVec2 { x: p.x + scale*(normal.x as f32), 
                              y: p.y + scale*(-normal.y as f32) };
            let pb = ImVec2 { x: p.x + scale*(-normal.x as f32),
                              y: p.y + scale*(-(-normal.y) as f32) };

            let p1 = model.world_to_screen(*p1); 
            let p2 = model.world_to_screen(*p2);

            ImDrawList_AddLine(draw_list,
                   ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y },
                   ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
                   c3, 4.0);

            ImDrawList_AddLine(draw_list,
                   ImVec2 { x: pos.x + pa.x, y: pos.y + pa.y },
                   ImVec2 { x: pos.x + pb.x, y: pos.y + pb.y },
                   c4, 2.0);
        }

        if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
            if let Some(o) = &model.adding_object {
                if igSelectable(const_cstr!("signal").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    println!("Add signal at {:?}", model.adding_object.unwrap());
                }
                if igSelectable(const_cstr!("detector").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    println!("Add detector at {:?}", model.adding_object.unwrap());
                }
            }
            if let Some(pt) = &model.editing_node {
                let pt = *pt;

                if igSelectable(const_cstr!("Nodetype OpenEnd").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    model.document.node_data.insert(pt,NDType::OpenEnd);
                    println!("NODE DATA {:?}", model.document.node_data);
                    model.editing_node = None;
                    model.refresh();
                }
                if igSelectable(const_cstr!("Nodetype BufferStop").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    model.document.node_data.insert(pt,NDType::BufferStop);
                    println!("NODE DATA {:?}", model.document.node_data);
                    model.editing_node = None;
                    model.refresh();
                }
            }

            igEndPopup();
        }

        if !igIsPopupOpen(const_cstr!("ctx").as_ptr()) {
            model.adding_object = None; // Cancelled by GUI
            model.editing_node = None; // Cancelled by GUI
        }

        ImDrawList_PopClipRect(draw_list);
    }
}

