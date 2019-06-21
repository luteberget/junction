use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::text::*;
use const_cstr::const_cstr;
use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};
use matches::matches;


use crate::pt::*;
use crate::symset::*;
use crate::topology::*;

pub type PtC = (f32,f32);

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
pub enum Tool { Scroll, Draw, Modify, Erase }

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Document {
    pieces :SymSet<Pt>,
    // TODO symbols, node types, etc.
    // TODO how to do naming etc in dispatches / movements.
    railway :Option<Railway>,

    // Objects are stored by their position. 
    // The rest of the information is derived
    objects :Vec<(PtC,Object)>,
    node_data :HashMap<Pt, NDType>, // copied into railway when topology has changed
}

#[derive(Debug)]
pub struct SchematicCanvas {
    document :Document,
    tool: Tool,
    selection : (HashSet<(Pt,Pt)>, HashSet<usize>),
    scale: Option<usize>,
    translate :Option<ImVec2>,
    adding_line :Option<Pt>,
    adding_object: Option<((f32,f32),(Pt,Pt))>,  // Pt-continuous
    editing_node: Option<Pt>,
    selecting_rectangle: Option<ImVec4>,
    dragging_objects :Option<PtC>,
}

// TODO model editor state like this
enum CurrentAction {
    None,
    DrawingLine(Pt),
    SelectObjectType(()),
    PlacingObject(PtC, (Pt,Pt)),
    EditingNode(Pt),
    EditingObject(usize),
    SelectingRectangle(ImVec4),
    DraggingObjectsDiscretely(PtC),
}

impl SchematicCanvas {

    pub fn new() -> Self {

        SchematicCanvas {
            document: Document {
                pieces: SymSet::new(),
                objects: Vec::new(),
                node_data: HashMap::new(),
                railway: None,
            },
            selection: (HashSet::new(), HashSet::new()),
            tool: Tool::Scroll,
            adding_line: None,
            scale: None,
            translate :None,//ImVec2{ x:0.0, y:0.0 },
            adding_object: None,
            editing_node: None,
            selecting_rectangle: None,
            dragging_objects: None,
        }
    }

    pub fn add_signal(&mut self, pt :PtC) {

        if let Some((p1,p2)) = self.closest_edge(pt) {
            let pt_on_edge = project_to_line(pt,(p1,p2));
            let normal = (pt.0 - pt_on_edge.0, pt.1 - pt_on_edge.1);
            if normal.0.abs() + normal.1.abs() > 0.0 {
                let normal_len = (normal.0 * normal.0 + normal.1 * normal.1).sqrt();
                let normal_normal = (normal.0 / normal_len, normal.1 / normal_len);
                let new_pt = (pt_on_edge.0 + normal_normal.0 * 0.25, 
                              pt_on_edge.1 + normal_normal.1 * 0.25);
                let angle = modu((normal.1.atan2(normal.0) / (2.0 * std::f32::consts::PI) * 8.0).round() as i8 + 2, 8);
                println!("signal at {:?}, tangent {:?}", pt, angle_v(angle));
                self.document.objects.push((new_pt, Object {
                    data: ObjectData::Signal,
                    tangent: angle_v(angle),
                    mileage: None,
                    distance: None,
                }));
            } else {
                eprintln!("Cannot add signal here?");
            }
        } else {
            eprintln!("Cannot add signal here?");
        }
    }

    pub fn add_detector(&mut self, pt :PtC) {
        // project it to the nearest point on th eline
        if let Some((p1,p2)) = self.closest_edge(pt) {
            let tangent = Pt { x: p2.x - p1.x, y: p2.y - p1.y };
            let pt = project_to_line(pt,(p1,p2)); // world coords
            println!("detector at {:?}, tangent {:?}", pt, tangent);
            self.document.objects.push((pt, Object {
                data: ObjectData::Detector,
                tangent: tangent,
                mileage: None,
                distance: None,
            }));
        }
    }

    pub fn refresh(&mut self) {
        self.document.railway = to_railway(self.document.pieces.clone(), 
                                           &mut self.document.node_data,
                                           50.0).ok();
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

    pub fn set_selection_rect(&mut self, a_in :PtC, b_in :PtC) {
        let a = (a_in.0.min(b_in.0), a_in.1.min(b_in.1));
        let b = (a_in.0.max(b_in.0), a_in.1.max(b_in.1));
        println!("Selecting {:?} {:?}", a, b);
        self.selection.0.clear();
        self.selection.1.clear();
        // select edges
        let doc = &mut self.document;
        let set = &mut self.selection.0;
        doc.pieces.iter(|p1,p2| {
            for e in &[p1,p2] {
                let p = (e.x as f32, e.y as f32);
                if a.0 <= p.0 && p.0 <= b.0 && a.1 <= p.1 && p.1 <= b.1 {
                    set.insert((*p1,*p2));
                }
            }
        });
        for (i,(p,_)) in self.document.objects.iter().enumerate() {
            if a.0 <= p.0 && p.0 <= b.0 && a.1 <= p.1 && p.1 <= b.1 {
                self.selection.1.insert(i);
            }
        }

        println!("Selected pieces {:?} ",self.selection.0);
    }

    pub fn set_selection_closest(&mut self, pt :PtC, threshold :f32) {
        let t2 = threshold*threshold;
        let mut near = Vec::new();
        pub enum Thing { Object(usize), Edge(Pt,Pt) }
        self.document.pieces.iter(|p1,p2| {
            let dist = dist_to_line_sqr(pt,(*p1,*p2));
            if dist < t2 { near.push((dist,Thing::Edge(*p1,*p2))); }
        });
        for (i,(q,_)) in self.document.objects.iter().enumerate() {
            let dist = lsqr(pt,*q);
            if dist < t2 { near.push((dist,Thing::Object(i))); }
        }
        let sel = near.into_iter().fold(None, |min, x| match min {
            None => Some(x),
            Some(y) => Some(if x.0 < y.0 { x } else { y }),
        });
        self.selection.0.clear();
        self.selection.1.clear();
        match sel {
            Some((_,Thing::Edge(p1,p2))) => { self.selection.0.insert((p1,p2)); },
            Some((_,Thing::Object(i))) => { self.selection.1.insert(i); },
            _ => {},
        }
    }

    pub fn realize_discrete_drag(&mut self) -> bool {
        let vx = self.dragging_objects.unwrap().0;
        let mut changed = false;
        if vx.abs() >= 1.0 {
            self.move_selected_pieces(Pt { x: vx.signum() as i32, y: 0 });
            self.move_selected_objects((vx.signum(), 0.0));
            self.dragging_objects.as_mut().unwrap().0 -= vx.signum()*1.0;
            changed = true;
        }
        let vy = self.dragging_objects.unwrap().1;
        if vy.abs() >= 1.0 {
            self.move_selected_pieces(Pt { x: 0, y: vy.signum() as i32 });
            self.move_selected_objects((0.0, vy.signum()));
            self.dragging_objects.as_mut().unwrap().1 -= vy.signum()*1.0;
            changed = true;
        }
        changed
    }

    pub fn move_selected_pieces(&mut self, d: Pt) {
        println!(" MOVE by {:?} \n\n\n",d);
        // Edges
        let mut new_selection = HashSet::new();
        for (p1,p2) in self.selection.0.drain() {
            self.document.pieces.remove((p1,p2));
            let p1 = Pt { x: p1.x + d.x, y: p1.y + d.y };
            let p2 = Pt { x: p2.x + d.x, y: p2.y + d.y };
            new_selection.insert((p1,p2));
        }

        for e in &new_selection { self.document.pieces.insert(*e); }

        let node_data = self.document.node_data.clone();
        let node_data = node_data.into_iter()
            .map(|(pt,x)| (Pt { x: pt.x + d.x, y: pt.y + d.y }, x)).collect();
        self.document.node_data = node_data;

        self.selection.0 = new_selection;
        println!(" MOVE DONE \n\n\n");
    }

    pub fn move_selected_objects(&mut self, d :PtC) {

        // Objects
        for i in self.selection.1.iter() {
            let i = *i;
            let old = self.document.objects[i].0;
            let new = (old.0+d.0,old.1+d.1);
            self.document.objects[i].0 = new;
        }
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
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("Ers").as_ptr(), matches!(&model.tool, Tool::Erase)) {
            model.tool = Tool::Erase;
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

        // Handle mouse wheel
        model.scale = Some( (model.scale.unwrap_or(35) as f32 + 3.0*(*io).MouseWheel).max(20.0).min(150.0).round() as _ );

        // Handle normal keys
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
            if igIsKeyPressed('F' as _, false) {
                model.tool = Tool::Erase; 
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

        if let Tool::Erase = &model.tool {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                // find nearest edge and erase it
                let loc = model.screen_to_world_cont(pointer_incanvas);
                if let Some(edge) = model.closest_edge(loc) {
                    println!("ERASE {:?}", edge);
                    model.document.pieces.remove(edge);
                    model.refresh();
                }
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


            let special_key = (*io).KeyCtrl | (*io).KeyAlt | (*io).KeySuper;
            if (igIsItemActive() || !igIsAnyItemActive()) && !special_key {
                if igIsKeyPressed('X' as _, false) { 
                    println!("delete {:?}", model.selection);
                    for e in model.selection.0.drain() {
                        model.document.pieces.remove(e);
                    }
                    let mut objs = model.selection.1.drain().collect::<Vec<_>>();
                    objs.sort_by_key(|i| -(*i as isize));
                    for i in objs {
                        model.document.objects.remove(i);
                    }
                    model.refresh();
                }
            }
            //
            // MOVE THINGS
            if igIsMouseDragging(1,-1.0) {
                // MOVE THINGS
                let discrete = !model.selection.0.is_empty();
                let w1 = model.screen_to_world_cont(ImVec2 { x: 0.0, y: 0.0 });
                let w2 = model.screen_to_world_cont((*io).MouseDelta);
                let d = (w2.0 - w1.0, w2.1 - w1.1);
                if discrete {
                    if model.dragging_objects.is_none() { 
                        model.dragging_objects = Some((0.0,0.0));
                    }

                    let v = model.dragging_objects.as_mut().unwrap();
                    *v = (v.0 + d.0, v.1 + d.1);
                    if model.realize_discrete_drag() { model.refresh(); }

                } else {
                    model.move_selected_objects(d);
                }
            } else {
                model.dragging_objects = None;
            }

            if igIsMouseDragging(0,-1.0) {
                let a = (*io).MouseClickedPos[0];
                let a = ImVec2 { x: a.x - pos.x, y: a.y - pos.y };
                let delta = igGetMouseDragDelta_nonUDT2(0,-1.0);
                let b = ImVec2 { x: a.x + delta.x, y: a.y + delta.y };
                //println!("dragging {:?} {:?}", (*io).MouseClickedPos, delta);
                ImDrawList_AddRect(draw_list,
                                   ImVec2 { x: pos.x + a.x, y: pos.y + a.y },
                                   ImVec2 { x: pos.x + b.x, y: pos.y + b.y },
                                   c2,0.0,0,1.0);
                model.selecting_rectangle = Some(ImVec4 { x: a.x, y: a.y, z: b.x, w: b.y });
            } else {
                if let Some(rect) = &model.selecting_rectangle {
                    println!("SELECTING {:?}", rect); 
                    let a = model.screen_to_world_cont(ImVec2 { x: rect.x, y: rect.y });
                    let b = model.screen_to_world_cont(ImVec2 { x: rect.z, y: rect.w });
                    model.set_selection_rect(a,b);
                    model.selecting_rectangle = None;
                }
            }

            if igIsItemHovered(0) && igIsMouseClicked(0,false) {
                model.set_selection_closest(model.screen_to_world_cont(pointer_incanvas), 
                                            5.0 / model.scale.unwrap_or(35) as f32);
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
                    let col = if model.selection.0.contains(&(*p,*q)) { c3 } else { c2 };
                    line(col, &model.world_to_screen(*p), &model.world_to_screen(*q));
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

        // Draw objects
        for (obj_i,(p,o)) in model.document.objects.iter().enumerate() {
            let p = model.world_to_screen_cont(*p);
            let selected = model.selection.1.contains(&obj_i);
            match o.data {
                ObjectData::Signal => {
                    let scale = 5.0;
                    let tangent = o.tangent;
                    let c = if selected { c5 } else { c4 };
                    let normal = Vc { x: -tangent.y, y: tangent.x };

                    let p2 = ImVec2 { x: p.x + scale*(tangent.x as f32),
                                      y: p.y + scale*(-tangent.y as f32) };
                    let p3 = ImVec2 { x: p2.x + scale*(tangent.x as f32),
                                      y: p2.y + scale*(-tangent.y as f32) };

                    let pa = ImVec2 { x: p.x + scale*(normal.x as f32),
                                      y: p.y + scale*(-normal.y as f32) };
                    let pb = ImVec2 { x: p.x + scale*(-normal.x as f32),
                                      y: p.y + scale*(normal.y as f32) };

                    ImDrawList_AddLine(draw_list,
                           ImVec2 { x: pos.x + p.x, y: pos.y + p.y },
                           ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
                           c, 2.0);
                    ImDrawList_AddLine(draw_list,
                           ImVec2 { x: pos.x + pa.x, y: pos.y + pa.y },
                           ImVec2 { x: pos.x + pb.x, y: pos.y + pb.y },
                           c, 2.0);
                    ImDrawList_AddCircle(draw_list,
                           ImVec2 { x: pos.x + p3.x, y: pos.y + p3.y },
                           scale, c, 8, 2.0);

                },
                ObjectData::Detector => {
                    let c = if selected { c5 } else { c4 };
                    let scale = 5.0;
                    let tangent = o.tangent;
                    let normal = Vc { x: -tangent.y, y: tangent.x };
                    let pa = ImVec2 { x: p.x + scale*(normal.x as f32), 
                                      y: p.y + scale*(-normal.y as f32) };
                    let pb = ImVec2 { x: p.x + scale*(-normal.x as f32),
                                      y: p.y + scale*(-(-normal.y) as f32) };

                    ImDrawList_AddLine(draw_list,
                           ImVec2 { x: pos.x + pa.x, y: pos.y + pa.y },
                           ImVec2 { x: pos.x + pb.x, y: pos.y + pb.y },
                           c, 2.0);
                },
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
            if let Some((o,_)) = &model.adding_object {
                let o = *o;
                if igSelectable(const_cstr!("signal").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    println!("Add signal at {:?}", o);
                    //model.document.objects.push((o.0, Object::Signal(AB::A)));
                    model.add_signal(o);
                }
                if igSelectable(const_cstr!("detector").as_ptr(), false, 0 as _, ImVec2 { x: 0.0, y: 0.0 }) {
                    println!("Add detector at {:?}", o);
                    model.add_detector(o);
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

