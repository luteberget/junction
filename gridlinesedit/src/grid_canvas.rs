use imgui_sys_bindgen::sys::*;
use const_cstr::const_cstr;
use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};


use crate::pt::*;
use crate::symset::*;

// 
//
// Railway
//

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum NDType { OpenEnd, BufferStop, Cont, Sw(Side), Err }

#[derive(Debug,Copy,Clone)]
pub enum AB { A, B }

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum Port { End, ContA, ContB, Left, Right, Trunk, Err }

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum Side { Left, Right }

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    pub fn to_port(&self) -> Port {
        match self {
            Side::Left => Port::Left,
            Side::Right => Port::Right,
        }
    }

    pub fn to_rotation(&self) -> i8 {
        match self {
            Side::Left => 1,
            Side::Right => -1,
        }
    }
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct Railway {
    pub locations: Vec<(Pt, NDType, Vc)>,
    pub tracks: Vec<((usize,Port),(usize,Port), f64)>,
}

pub fn to_railway(mut pieces :SymSet<Pt>, def_len :f64) -> Result<Railway, ()>{
    // while edges:
    // 1. pick any edge from bi-indexed set
    // 2. follow edge to nodes, removing nodes from set
    // 3. create track there, put ends into another set
    //let mut pieces = SymSet::from_iter(ls);
    let mut tracks :Vec<(Pt,Pt,f64)> = Vec::new();
    let mut locs :HashMap<Pt, Vec<((usize,AB),Pt)>> = HashMap::new();
    println!("PIECES {:?}", pieces);
    while let Some((p1,p2)) = pieces.remove_any() {
        println!("adding track starting in {:?}", (p1,p2));
        let mut length = def_len;
        let (mut a, mut b) = ((p1,p2),(p2,p1));
        drop(p1); drop(p2);
        let mut extend = |p :&mut (Pt,Pt), other :Pt| {
            loop {
                println!("Extending from {:?}", p.0);
                if locs.contains_key(&p.0) || p.0 == other  { break; /* Node exists. */ }
                if let Some(n) = pieces.remove_single(p.0) {
                    *p = (n,p.0);
                    length += def_len;
                } else {
                    break; // Either no more nodes along the path,
                           // or the path splits. In any case, add node here.
                }
            }
        };

        extend(&mut a,b.0); println!("Done extending {:?}", a); 
        extend(&mut b,a.0); println!("Done extending {:?}", b); 
        let track_idx = tracks.len();
        tracks.push((a.0,b.0,length));
        locs.entry(a.0).or_insert(Vec::new()).push(((track_idx, AB::A), a.1));
        locs.entry(b.0).or_insert(Vec::new()).push(((track_idx, AB::B), b.1));
        println!("after iter PIECES {:?}", pieces);
    }
        // Now we have tracks from node locations A/B
    // and locations with each track's incoming angles
    // We want to transform into 
    // 1. list of locations with node type and corresponding orientation,
    //      LIdx -> (Pt, NDType, Vc)
    // 2. Tracks with start/end links to locations and corresponding PORTS.
    //      TIdx -> ((LIdx,Port),(LIdx,Port),Length)

    println!("SO FAR SO GOOD");
    println!("{:?}", locs);
    println!("{:?}", tracks);

    let mut tp :Vec<(Option<(usize,Port)>,Option<(usize,Port)>,f64)> =
        tracks.into_iter().map(|(_,_,l)| (None,None,l)).collect();
    let mut settr = |(i,ab) : (usize,AB), val| match ab {
        AB::A => tp[i].0 = val,
        AB::B => tp[i].1 = val,
    };
    let mut locx :Vec<(Pt, NDType, Vc)> = Vec::new();
    for (l_i,(p,conns)) in locs.into_iter().enumerate() {
        match conns.as_slice() {
            [(t,q)] => {
                settr(*t,Some((l_i, Port::End)));
                locx.push((p, NDType::OpenEnd, pt_sub(*q,p)));
            },
            [(t1,q1),(t2,q2)] => {
                settr(*t1,Some((l_i,Port::ContA))); settr(*t2,Some((l_i,Port::ContB)));
                locx.push((p, NDType::Cont, pt_sub(*q1,p)));
            },
            [(t1,q1),(t2,q2),(t3,q3)] => {
                let track_idxs = [*t1,*t2,*t3];
                let qs = [*q1,*q2,*q3];
                let angle = [v_angle(pt_sub(*q1,p)), v_angle(pt_sub(*q2,p)), v_angle(pt_sub(*q3,p))];
                let permutations = &[[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]];
                let mut found = false;
                for pm in permutations {
                    let angle_diff = modu((angle[pm[2]]-angle[pm[1]]),8);
                    // p.0 is trunk, p.1 is straight, and p.2 is branch.
                    if !(angle[pm[0]] % 4 == angle[pm[1]] % 4 &&
                         angle_diff == 1 || angle_diff == 7) { 
                        continue; 
                    }
                    else { found = true; }

                    println!("SWITCH {} {} {}", 
                             angle[pm[0]], 
                             angle[pm[1]], 
                             angle[pm[2]]);
                    // TODO the side is not correct?
                    let side = if angle_diff == 1 { Side::Left } else { Side::Right };
                    settr(track_idxs[pm[0]],Some((l_i, Port::Trunk)));
                    settr(track_idxs[pm[1]],Some((l_i, side.opposite().to_port())));
                    settr(track_idxs[pm[2]],Some((l_i, side.to_port())));
                    locx.push((p, NDType::Sw(side), pt_sub(qs[pm[1]], p)));
                    break;
                }
                if !found { panic!("switch didn't work"); } // TODO add err values?
            },
            _ => unimplemented!(), // TODO
        }
    }

    let tp :Vec<((usize,Port),(usize,Port),f64)> = tp.into_iter()
        .map(|(a,b,l)| (a.unwrap(),b.unwrap(),l)).collect();

        Ok(Railway {
        locations: locx,
        tracks: tp,
    })
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct SchematicCanvas {
    pieces :SymSet<Pt>,
    // TODO symbols, node types, etc.
    // TODO how to do naming etc in dispatches / movements.
    railway :Option<Railway>,

    #[serde(skip)]
    scale: Option<usize>,
    #[serde(skip)]
    translate :Option<ImVec2>,
    #[serde(skip)]
    adding_line :Option<Pt>,
}

impl SchematicCanvas {
    pub fn new() -> Self {
        SchematicCanvas {
            pieces: SymSet::new(),
            railway: None,
            adding_line: None,
            scale: None,
            translate :None,//ImVec2{ x:0.0, y:0.0 },
        }
    }
        /// Converts and rounds a screen coordinate to the nearest point on the integer grid
    pub fn screen_to_world(&self, pt :ImVec2) -> Pt {
        let t = self.translate.unwrap_or(ImVec2{x:0.0,y:0.0});
        let s = self.scale.unwrap_or(35);
        let x =  (t.x + pt.x) / s as f32;
        let y = -(t.y + pt.y) / s as f32;
        Pt { x: x.round() as _ , y: y.round() as _ }
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
            serde_json::to_writer(json_file, c).expect("could not write to file");
            println!("Saved.{}",serde_json::to_string(c).expect("could not serialize"));
        }
        if (*io).KeyCtrl && igIsKeyPressed('L' as _, false) {
            let json_file = File::open(json_file_path).expect("file not found");
            *c = serde_json::from_reader(json_file).expect("error while reading json");
            println!("Loaded.\n{:?}",c);
        }
    }
}

pub fn schematic_canvas(size: &ImVec2, model: &mut SchematicCanvas) {
    unsafe {
        schematic_hotkeys(model);

        let io = igGetIO();
        let draw_list = igGetWindowDrawList();
        let pos = igGetCursorScreenPos_nonUDT2();
        let pos = ImVec2 { x: pos.x, y: pos.y };

        let c1 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 } );
        let c2 = igGetColorU32Vec4(ImVec4 { x: 0.2, y: 0.5, z: 0.95, w: 1.0 } );
        let c3 = igGetColorU32Vec4(ImVec4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 } );
        let c4 = igGetColorU32Vec4(ImVec4 { x: 0.8, y: 0.8, z: 0.8, w: 1.0 } );
        let c5 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.4, z: 0.8, w: 1.0 } );
        let c6 = igGetColorU32Vec4(ImVec4 { x: 0.6, y: 0.3, z: 0.33, w: 1.0 } );

        ImDrawList_AddRectFilled(draw_list,
                        pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y },
                        c1, 0.0, 0);
        igInvisibleButton(const_cstr!("grid_canvas").as_ptr(), *size);
        ImDrawList_PushClipRect(draw_list, pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y}, true);

        let pointer = (*io).MousePos;
        let pointer_incanvas = ImVec2 { x: pointer.x - pos.x, y: pointer.y - pos.y };
        let pointer_grid = model.screen_to_world(pointer_incanvas);

        let line = |c :ImU32,p1 :&ImVec2,p2 :&ImVec2| {
            ImDrawList_AddLine(draw_list,
                   ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y },
                   ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
                   c, 2.0);
        };

        // Drawing or adding line
        match (igIsItemHovered(0), igIsMouseDown(0), &mut model.adding_line) {
            (true, true, None)   => { model.adding_line = Some(pointer_grid); },
            (_, false, Some(pt)) => {
                for (p1,p2) in SchematicCanvas::route_line(*pt, pointer_grid) {
                    for (p1,p2) in unit_step_diag_line(p1, p2).zip(
                            unit_step_diag_line(p1, p2).skip(1)) {
                        println!("ADdding {:?} {:?}", p1,p2);
                        model.pieces.insert((p1,p2));
                    }
                }
                model.railway = to_railway(model.pieces.clone(), 50.0).ok();
                println!("Got new railway:");
                println!("{:#?}", &model.railway);
                model.adding_line = None;
            },
            _ => {},
        };

        // Draw permanent lines
        for (p,set) in &model.pieces.map {
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

        // Draw grid + highlight on closest point if hovering?
        let (lo,hi) = model.points_in_view(*size);
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                let pt = model.world_to_screen(Pt { x, y });
                ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + pt.x, y: pos.y + pt.y },
                                           3.0, c4, 4);
            }
        }

        // Draw nodes
        if let Some(r) = &model.railway {
            for (pt,typ,vc) in &r.locations {
                let p1 = model.world_to_screen(*pt);
                //let p2 = model.world_to_screen(Pt { x: pt.x + vc.x,
                //                                    y: pt.y + vc.y });
                match typ {
                    NDType::OpenEnd => {
                        ImDrawList_AddCircleFilled(draw_list, ImVec2 { x: pos.x + p1.x, 
                            y: pos.y + p1.y }, 6.0, c3, 8);
                    },
                    NDType::BufferStop => {},
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
                                               c6);
                        
                    },
                    NDType::Err => {
                    },
                }
            }
        }

        ImDrawList_PopClipRect(draw_list);
    }
}

