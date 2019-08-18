use std::collections::{BTreeSet, BTreeMap,HashMap, VecDeque};
use nalgebra_glm as glm;
use crate::model::*;
use crate::util::*;
use crate::objects::*;
use ordered_float::OrderedFloat;

#[derive(Debug)]
pub struct Topology {
    pub tracks : Vec<(f64,(Pt,Port),(Pt,Port))>,
    pub locations : HashMap<Pt,(NDType,Vc)>,
    pub trackobjects : HashMap<usize,Vec<(f64,PtA, Function,Option<AB>)>>,
    pub interval_lines :Vec<Vec<(OrderedFloat<f64>,PtC)>>,
}

impl Topology {
    pub fn interval_map(&self, track_idx: usize, start :f64, end :f64) -> Vec<PtC> {
        let lines = &self.interval_lines[track_idx];
        let mut output = Vec::new();

        // start with a binary search to find the first vertex _before_ the start.
        let mut i = {
            match lines.binary_search_by_key(&OrderedFloat(start), |&(l,_pt)| l) {
                Err(i) if i > 0 => {
                    // lerp the previous segment
                    let ((OrderedFloat(pos_a),pt_a),
                         (OrderedFloat(pos_b),pt_b)) = (&lines[i-1],&lines[i]);
                    output.push(glm::lerp(pt_a,pt_b, ((start - pos_a) / (pos_b - pos_a)) as f32));
                    i
                },
                Ok(i)|Err(i) => i,
            }
        };

        // Internal segments
        while i < lines.len() && !(lines[i].0 > OrderedFloat(end)) {
            output.push(lines[i].1);
            i += 1;
        }

        // lerp final segment
        if i > 0 && i < lines.len() && lines[i].0 > OrderedFloat(end) {
            let ((OrderedFloat(pos_a),pt_a),
                 (OrderedFloat(pos_b),pt_b)) = (&lines[i-1],&lines[i]);
            output.push(glm::lerp(pt_a,pt_b, ((end - pos_a) / (pos_b - pos_a)) as f32));
        }

        output
    }
}


//pub fn convert(model :&Model, def_len :f64) -> Result<(Tracks,Locations,TrackObjects,im::HashMap<Pt,NDType>), ()>{
pub fn convert(model :&Model, def_len :f64) -> Result<Topology, ()>{


    let mut tracks :Vec<(Pt,Pt,f64)> = Vec::new();
    let mut locs :HashMap<(i32,i32), Vec<((usize,AB),Pt)>> = HashMap::new();
    let mut interval_lines = Vec::new();

    let mut pieces = SymSet::new();
    for (a,b) in model.linesegs.iter() {
        pieces.insert(((a.x,a.y),(b.x,b.y)));
    }

    let mut piece_map : HashMap<((i32,i32),(i32,i32)), (usize, f64, f64)> = HashMap::new();
    let mut trackobjects = HashMap::new();
    while let Some((p1,p2)) = pieces.remove_any() {
        let mut list = VecDeque::new();
        list.push_back((p1,p2));

        let mut length = def_len;
        let (mut a, mut b) = ((p1,p2),(p2,p1));
        drop(p1);drop(p2);

        let mut extend = |p :&mut ((i32,i32),(i32,i32)), other: (i32,i32)| {
            loop {
                if locs.contains_key(&p.0) || p.0 == other { break; }
                if let Some(n) = pieces.remove_single(p.0) {
                    if list[0].0 == n {
                        list.push_front((p.0,n));
                    } else if list[0].0 == p.0 {
                        list.push_front((n,p.0));
                    } else if list[list.len()-1].1 == n {
                        list.push_back((n,p.0));
                    } else if list[list.len()-1].1 == p.0 {
                        list.push_back((p.0,n));
                    } else { panic!(); }

                    *p = (n,p.0);
                    length += def_len;

                } else {
                    break;
                }
            }
        };

        extend(&mut a, b.0);
        extend(&mut b, a.0);
        let track_idx = tracks.len();
        tracks.push((to_vec(a.0),to_vec(b.0),length));
        locs.entry(a.0).or_insert(Vec::new()).push(((track_idx, AB::A), to_vec(a.1)));
        locs.entry(b.0).or_insert(Vec::new()).push(((track_idx, AB::B), to_vec(b.1)));


        //println!("List {:?}", list);
        let mut l = 0.0;
        let mut interval_map = Vec::new();
        for (a,b) in list.iter().cloned() {
            piece_map.insert((a,b), (tracks.len()-1, l, def_len));
            interval_map.push((OrderedFloat(l),glm::vec2(a.0 as f32 ,a.1 as f32)));
            l += def_len;
        }
        let last_pt = list[list.len()-1].1;
        interval_map.push((OrderedFloat(l),glm::vec2(last_pt.0 as f32, last_pt.1 as f32)));
        interval_lines.push(interval_map);
        trackobjects.insert(tracks.len()-1, Vec::new());
    }

    fn get_dir_from_side((a,b) :&(Pt,Pt), pt :PtC) -> AB {
        let (pt_on_line,_param) = project_to_line(pt, glm::vec2(a.x as _, a.y as _),
                                                      glm::vec2(b.x as _, b.y as _));
        let tangent = glm::vec2(b.x as f32 - a.x as f32, b.y as f32 - a.y as f32);
        let normal = glm::vec2(-tangent.y, tangent.x);
        let a = glm::angle(&(pt_on_line - pt), &normal);
        if glm::angle(&(pt_on_line - pt), &normal) > glm::half_pi() {
            AB::B
        } else { AB::A }
    }

    let get_from_piece_map = |a :(i32,i32), b :(i32,i32)| -> Option<&(usize,f64,f64)> {
        let get1 = piece_map.get(&(a,b));
        if get1.is_some() { return get1; }
        piece_map.get(&(b,a));
        None
    };

    for &(id,Object { symbol }) in model.objects.iter() {
        if let Some((pt,param,_)) = model.get_closest_lineseg(symbol.loc) {
            if let Some((track_idx,pos_start,length)) = get_from_piece_map((pt.0.x,pt.0.y), (pt.1.x,pt.1.y)) {
                let pos = pos_start + (param as f64) *length;
                let (func,dir) = match symbol.shape {
                    Shape::Detector => (Function::Detector, None),
                    Shape::Signal   => (Function::MainSignal, Some(get_dir_from_side(&pt,symbol.loc))),
                };
                trackobjects.entry(*track_idx).or_insert(Vec::new()).push((pos, id, func, dir));
            } else {
                println!("WARNING: object trackside position error.");
            }
        } else {
            println!("WARNING: object outside track network.");
        }
    }

    let mut tp : Vec<(Option<(Pt,Port)>, Option<(Pt,Port)>, f64)> =
        tracks.into_iter().map(|(_,_,l)| (None,None,l)).collect();

    let mut settr = |(i,ab) :(usize,AB), val| match ab {
        AB::A => tp[i].0 = val,
        AB::B => tp[i].1 = val,
    };

    let mut locx :HashMap<Pt,(NDType,Vc)> = HashMap::new();

    for (p,conns) in locs.into_iter() {
        let p = to_vec(p);
        match conns.as_slice() {
            [(t,q)] => {
                settr(*t, Some((p, Port::End)));
                locx.insert(p,(NDType::OpenEnd, *q - p));
            },
            [(t1,q1),(t2,q2)] => {
                settr(*t1, Some((p, Port::ContA)));
                settr(*t2, Some((p, Port::ContB)));
                locx.insert(p,(NDType::Cont, *q1 - p));
            },
            [(t1,q1),(t2,q2),(t3,q3)] => {
                let track_idxs = [*t1,*t2,*t3];
                let qs = [*q1,*q2,*q3];
                let angle = [v_angle(p-*q1), v_angle(p-*q2), v_angle(p-*q3)];
                let permutations = &[[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]];
                let mut found = false;
                for pm in permutations {
                    let angle_diff = modu(angle[pm[2]] - angle[pm[1]], 8);
                    if !(angle[pm[0]] % 4 == angle[pm[1]] % 4 && (angle_diff == 1 || angle_diff == 7)) {
                        continue;
                    } else {
                        found = true;
                    }

                    let side = if angle_diff == 1 { Side::Left } else { Side::Right };
                    settr(track_idxs[pm[0]], Some((p, Port::Trunk)));
                    settr(track_idxs[pm[1]], Some((p, side_to_port(opposite(side)))));
                    settr(track_idxs[pm[2]], Some((p, side_to_port(side))));
                    locx.insert(p,(NDType::Sw(side), qs[pm[1]] - p));
                    break;
                }
                if !found { panic!("error in switch"); } // TODO report err
            },
            more if more.len() == 4 => {
                let mut pairs = [None,None,None,None];
                for (t,q) in more {
                    let angle = modu(v_angle(p-*q), 4) as usize;
                    match pairs[angle] {
                        None => { pairs[angle] = Some(Err((*t,*q))); },
                        Some(Err((t0,q0))) => { pairs[angle] = Some(Ok(((t0,q0),(*t,*q)))); },
                        _ => panic!(), // TODO report err
                    };
                }
                let mut n = 0;
                let mut maindir = None;
                for x in &pairs {
                    match x {
                        None => {},
                        Some(Ok(((t1,q1),(t2,q2)))) => {
                            // OK
                            if n == 0 { 
                                maindir = Some(*q1 - p); 
                            }
                            settr(*t1, Some((p, Port::Cross(AB::A,n))));
                            settr(*t2, Some((p, Port::Cross(AB::B,n))));

                            n += 1;
                        },
                        Some(Err(_)) => { panic!() }, // TODO report err
                    };
                }

                if n == 2 {
                    locx.insert(p, (NDType::Crossing, maindir.unwrap()));
                }
            },
            _ => {
                locx.insert(p,(NDType::Err, glm::zero()));
            },
        };
    }


    Ok(
        Topology {
            tracks: tp.into_iter().map(|(a,b,l)| (l, a.unwrap(), b.unwrap())).collect(),
            locations: locx,
            trackobjects: trackobjects,
            interval_lines: interval_lines, 
        }
    )
}

fn modu(a :i8, b:i8) -> i8 { (a % b + b ) % b }

fn v_angle(v :Vc) -> i8 {
    match (v.x.signum(),v.y.signum()) {
        ( 1, 0) => 0,
        ( 1, 1) => 1,
        ( 0, 1) => 2,
        (-1, 1) => 3,
        (-1, 0) => 4,
        (-1,-1) => 5,
        ( 0,-1) => 6,
        ( 1,-1) => 7,
        _ => panic!()
    }
}


#[derive(Debug,Clone)]
pub struct SymSet<T:Ord+Copy> {
    pub map :BTreeMap<T, BTreeSet<T>>,
}

impl<T:Ord+Copy> SymSet<T> {
    pub fn new() -> Self { SymSet { map: BTreeMap::new() } }

    pub fn iter(&self, mut f :impl FnMut(&T,&T)) {
        for (a,set) in self.map.iter() {
            for b in set {
                if !( a > b) {
                    f(a,b);
                }
            }
        }
    }

    pub fn insert(&mut self, pt :(T,T)) -> bool {
        let r1 = self.map.entry(pt.0).or_insert(BTreeSet::new()).insert(pt.1);
        let r2 = self.map.entry(pt.1).or_insert(BTreeSet::new()).insert(pt.0);
        if r1 != r2 { panic!(); }
        r1
    }

    pub fn remove(&mut self, pt :(T,T)) -> bool {
        let r1 = self.remove_oneway((pt.0,pt.1));
        let r2 = self.remove_oneway((pt.1,pt.0));
        if r1 != r2 { panic!(); }
        if r1 && self.map[&pt.0].is_empty() { self.map.remove(&pt.0); }
        if r2 && self.map[&pt.1].is_empty() { self.map.remove(&pt.1); }
        r1
    }

    fn remove_oneway(&mut self, pt :(T,T)) -> bool {
        self.map.get_mut(&pt.0).map(|s| s.remove(&pt.1)).unwrap_or(false)
    }

    pub fn contains(&self, val :(T,T)) -> bool {
        self.map.get(&val.0).map(|v| v.contains(&val.1)) == Some(true)
    }

    pub fn get_any(&self) -> Option<(T,T)> {
        let (e1,set) = self.map.iter().nth(0)?;
        let e2 = set.iter().nth(0)?;
        Some((*e1,*e2))
    }

    pub fn remove_any(&mut self) -> Option<(T,T)> {
        let elem = self.get_any()?;
        self.remove(elem);
        Some(elem)
    }

    pub fn remove_single(&mut self, val :T) -> Option<T> {
        let set = self.map.get_mut(&val)?;
        let other = *set.iter().nth(0)?;
        if set.len() != 1 { return None; }
        self.remove((val,other));
        Some(other)
    }

    pub fn from_iter(x :impl IntoIterator<Item = (T,T)>) -> Self {
        let mut s = SymSet::new();
        for i in x.into_iter() { s.insert(i); }
        s
    }
}

