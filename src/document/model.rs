use nalgebra_glm as glm;
use crate::document::objects::*;
use crate::document::infview::*;
use crate::util::*;
use ordered_float::OrderedFloat;
use serde::{Serialize,Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(Serialize,Deserialize)]
pub enum Side {
    Left, Right
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    pub fn as_port(&self) -> Port {
        match self {
            Side::Left => Port::Right,
            Side::Right => Port::Left,
        }
    }

    pub fn as_switch_position(&self) -> rolling::input::staticinfrastructure::SwitchPosition {
        match self {
            Side::Left => rolling::input::staticinfrastructure::SwitchPosition::Left,
            Side::Right => rolling::input::staticinfrastructure::SwitchPosition::Right,
        }
    }
}

pub type Pt = glm::I32Vec2;
pub type PtA = glm::I32Vec2;
pub type PtC = glm::Vec2;
pub type Vc = Pt;


#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct Vehicle {
    pub name :String,
    pub length: f32,
    pub max_acc :f32,
    pub max_brk :f32,
    pub max_vel :f32,
}

#[derive(Debug,Copy,Clone, PartialEq, Eq)]
#[derive(Serialize,Deserialize)]
pub enum CrossingType { 
    Crossover,
    SingleSlip(Side), // LEft means switching left when traveling with increasing X coord.
    DoubleSlip,
}

#[derive(Debug,Copy,Clone, PartialEq, Eq)]
#[derive(Serialize,Deserialize)]
pub enum NDType { OpenEnd, BufferStop, Cont, Sw(Side), Crossing(CrossingType), Err }
// TODO crossing switchable, crossing orthogonal?, what settings does a crossing have?
// Assuming non-switched crossing for now.

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum Port { End, ContA, ContB, Left, Right, Trunk, Err, Cross(AB,usize) }
// Crossing has AB as different sides of opposing ports, and usize as the different pairs of edges

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum AB { A, B }

impl AB {
    pub fn other(&self) -> AB {
        match self {
            AB::A => AB::B,
            AB::B => AB::A,
        }
    }
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub enum Command {
    Train { route :usize, vehicle :usize },
    Route { route: usize },
}

#[derive(Serialize,Deserialize)]
#[derive(Debug, Default, Clone)]
pub struct Dispatch(pub Vec<(f64,Command)>);

impl Dispatch {
    pub fn insert(&mut self, t :f64, cmd :Command) {
        let idx = match self.0.binary_search_by_key(&OrderedFloat(t), 
                    |(t,_)| OrderedFloat(*t)) { Ok(i) | Err(i) => i };
        self.0.insert(idx, (t, cmd));
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct PlanSpec {
    x :usize,
}


#[derive(Clone, Default)]
#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct Model {
    pub linesegs: im::HashSet<(Pt,Pt)>,
    pub objects: im::HashMap<PtA, Object>,
    pub node_data: im::HashMap<Pt, NDType>,
    pub vehicles :im::Vector<Vehicle>,
    pub dispatches :im::Vector<Dispatch>,
    pub plans :im::Vector<PlanSpec>,
}


#[derive(Hash,PartialEq,Eq)]
#[derive(Copy,Clone)]
#[derive(Debug)]
pub enum Ref {
    Node(Pt),
    LineSeg(Pt,Pt),
    Object(PtA),
}

fn closest_pts(pt :PtC) -> [(Pt,Pt);2] {
    let x_lo = pt.x.floor() as i32;
    let x_hi = pt.x.ceil()  as i32;
    let y_lo = pt.y.floor() as i32;
    let y_hi = pt.y.ceil()  as i32;
    
    [
        (glm::vec2(x_lo,y_lo),glm::vec2(x_hi,y_hi)),
        (glm::vec2(x_lo,y_hi),glm::vec2(x_hi,y_lo)),
    ]
}

pub fn corners(pt :PtC) -> Vec<Pt> {
    use itertools::iproduct;
    use nalgebra_glm::vec2; 
    iproduct!(
        [0.0,1.0].iter().map(|d| (pt.x + d).floor() as i32),
        [0.0,1.0].iter().map(|d| (pt.y + d).floor() as i32))
        .map(|(x,y)| vec2(x,y)).collect()
}

impl Model {
    pub fn empty() -> Self { Default::default() }

    pub fn get_closest_object<'a>(&'a self, pt :PtC) -> Option<((&'a PtA,&'a Object),f32)> {
        // TODO performance
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        for (p,o) in self.objects.iter() {
            let d = glm::length2(&(unround_coord(*p) - pt));
            if d < dist_sqr {
                thing = Some((p,o));
                dist_sqr = d;
            }
        }
        thing.map(|o| (o,dist_sqr))
    }

    pub fn get_closest_lineseg(&self, pt :PtC) -> Option<((Pt,Pt),f32,(f32,f32))> {
        // TODO performance
        let (mut thing,mut dist_sqr,mut next_dist) = (None, std::f32::INFINITY, std::f32::INFINITY);
        for x1 in [pt.x.floor() as i32, (pt.x + 1.0).floor() as i32].iter().cloned() {
        for y1 in [pt.y.floor() as i32, (pt.y + 1.0).floor() as i32].iter().cloned() {
        for x2 in [pt.x.floor() as i32, (pt.x + 1.0).floor() as i32].iter().cloned() {
        for y2 in [pt.y.floor() as i32, (pt.y + 1.0).floor() as i32].iter().cloned() {
            let l = (glm::vec2(x1,y1),glm::vec2(x2,y2));
            if self.linesegs.contains(&l) {
                let (d,param) = dist_to_line_sqr(pt, 
                                         glm::vec2(l.0.x as _ ,l.0.y as _ ), 
                                         glm::vec2(l.1.x as _ ,l.1.y as _ ));
                if d < dist_sqr {
                    next_dist = dist_sqr;
                    dist_sqr = d;
                    thing = Some((l,param));
                } else if d < next_dist {
                    next_dist = d;
                }
            }
        }
        }
        }
        }
        thing.map(|(tr,param)| (tr,param,(dist_sqr,next_dist)))
    }

    pub fn get_linesegs_in_rect(&self, a :PtC, b :PtC) -> Vec<(Pt,Pt)> {
        let mut r = Vec::new();
        for (p1,p2) in self.linesegs.iter() {
            if in_rect(glm::vec2(p1.x as f32, p1.y as f32), a, b) ||
               in_rect(glm::vec2(p2.x as f32, p2.y as f32), a, b) {
                   r.push((*p1,*p2));
            }
        }
        r
    }

    pub fn delete(&mut self, x :Ref) {
        match x {
            Ref::LineSeg(a,b) => { self.linesegs.remove(&(a,b)); },
            Ref::Node(a) => { self.node_data.remove(&a); },
            Ref::Object(p) => { self.objects.remove(&p); },
        }
    }


}

use std::collections::HashSet;
#[derive(Debug, PartialEq, Eq)]
pub enum EditClass {
    MoveObjects(HashSet<Ref>),
    CommandTime(usize,usize),
    VehicleName(usize),
    VehicleLen(usize),
    VehicleAcc(usize),
    VehicleBrk(usize),
    VehicleVel(usize),
}


pub struct Undoable<T, C> {
    stack :Vec<T>,
    pointer: usize,
    class :Option<C>,
}

impl<T : Clone + Default, C : Eq> Undoable<T,C> {
    pub fn info(&self) -> String {
        format!("Undo stack {}/{}", self.pointer, self.stack.len()-1)
    }

    pub fn new() -> Undoable<T,C> {
        Self::from(Default::default())
    }

    pub fn from(x :T) -> Undoable<T,C> {
        Undoable {
            stack: vec![x],
            pointer: 0,
            class: None,
        }
    }

    pub fn get(&self) -> &T {
        &self.stack[self.pointer]
    }

    pub fn set(&mut self, v :T, cl :Option<C>) {
        if cl.is_some() && self.class == cl {
            // replace the object if class matches
            self.stack[self.pointer] = v;
        } else {
            self.pointer += 1;
            self.stack.truncate(self.pointer);
            self.stack.push(v);
        }
        self.class = cl;
    }

    pub fn can_undo(&self) -> bool {
        self.pointer > 0
    }

    pub fn can_redo(&self) -> bool {
        self.pointer + 1 < self.stack.len()
    }

    pub fn undo(&mut self) -> bool {
        if self.pointer > 0 {
            self.pointer -= 1;
            self.class = None;
            true
        } else {
            false 
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.pointer + 1 < self.stack.len() {
            self.pointer += 1;
            self.class = None;
            true
        } else {
            false 
        }
    }

    pub fn override_edit_class(&mut self, cl :C) {
        self.class = Some(cl);
    }
}
