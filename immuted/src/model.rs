use nalgebra_glm as glm;
use crate::objects::*;
use crate::util::*;

pub use rolling::input::staticinfrastructure::SwitchPosition as Side;

pub fn opposite(side :Side) -> Side {
    match side {
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    }
}

pub fn side_to_port(side :Side) -> Port {
    match side {
        Side::Left => Port::Right,
        Side::Right => Port::Left,
    }
}

pub type Pt = glm::I32Vec2;
pub type PtA = glm::I32Vec2;
pub type PtC = glm::Vec2;
pub type Vc = Pt;

pub struct Undoable<T> {
    stack :Vec<T>,
    pointer: usize,
}

impl<T : Clone + Default> Undoable<T> {

    pub fn new() -> Undoable<T> {
        Undoable {
            stack: vec![Default::default()],
            pointer: 0,
        }
    }

    pub fn get(&self) -> &T {
        &self.stack[self.pointer]
    }

    pub fn set(&mut self, v :T) {
        self.pointer += 1;
        self.stack.truncate(self.pointer);
        self.stack.push(v);
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
            true
        } else {
            false 
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.pointer + 1 < self.stack.len() {
            self.pointer += 1;
            true
        } else {
            false 
        }
    }
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Object {
    pub symbol :Symbol,
    // TODO "semantics" (list of functions? main, distant, detector, etc.)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Function { MainSignal, Detector }

impl Object {
    pub fn has_function(&self, f: &Function) -> bool {
        // TODO replace with function list outside symbol/shape
        match (f, self.symbol.shape) {
            (Function::MainSignal, Shape::Signal) => true,
            (Function::Detector, Shape::Detector) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Vehicle {
}

#[derive(Debug,Copy,Clone)]
pub enum NDType { OpenEnd, BufferStop, Cont, Sw(Side), Crossing, Err }
// TODO crossing switchable, crossing orthogonal?, what settings does a crossing have?
// Assuming non-switched crossing for now.

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum Port { End, ContA, ContB, Left, Right, Trunk, Err, Cross(AB,usize) }
// Crossing has AB as different sides of opposing ports, and usize as the different pairs of edges

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum AB { A, B }

#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum Command {
    Train { start_node :Pt, vehicle :usize },
    Route { start_loc :PtA, end_loc :Option<PtA>, alternative :usize }
}

#[derive(Clone, Default)]
#[derive(Debug)]
pub struct Model {
    pub linesegs: im::HashSet<(Pt,Pt)>,
    pub objects: im::HashMap<PtA, Object>,
    pub node_data: im::HashMap<Pt, NDType>,
    pub vehicles :im::Vector<Vehicle>,
    pub dispatches :im::Vector<Vec<(f32,Command)>>,
}


#[derive(Hash,PartialEq,Eq)]
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

    pub fn get_rect(&self, a :PtC, b :PtC) -> Vec<Ref> {
        let (x_lo,x_hi) = (a.x.min(b.x), a.x.max(b.x));
        let (y_lo,y_hi) = (a.y.min(b.y), a.y.max(b.y));
        // TODO performance
        let mut r = Vec::new();
        for (a,b) in self.linesegs.iter() {
            let p1 = glm::vec2(a.x as f32,a.y as f32);
            let p2 = glm::vec2(b.x as f32,b.y as f32);
            if (x_lo <= p1.x && p1.x <= x_hi && y_lo <= p1.y && p1.y <= y_hi) ||
               (x_lo <= p2.x && p2.x <= x_hi && y_lo <= p2.y && p2.y <= y_hi) {
                   r.push(Ref::LineSeg(*a,*b));
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
