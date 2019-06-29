use nalgebra_glm as glm;
use crate::objects::*;
use crate::util::*;

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

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Vehicle {
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum NDType {
}

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
    Track(Pt,Pt),
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

impl Model {
    pub fn get_closest(&self, pt :PtC) -> Option<(Ref,(f32,f32))> {
        self.get_closest_lineseg(pt).map(|(a,b)| (Ref::Track(a.0,a.1),b))
    }

    pub fn get_closest_lineseg(&self, pt :PtC) -> Option<((Pt,Pt),(f32,f32))> {
        // TODO performance
        let (mut thing,mut dist_sqr,mut next_dist) = (None, std::f32::INFINITY, std::f32::INFINITY);
        println!("get closest from {:?}", pt);
        for x1 in [pt.x.floor() as i32, (pt.x + 1.0).floor() as i32].iter().cloned() {
        for y1 in [pt.y.floor() as i32, (pt.y + 1.0).floor() as i32].iter().cloned() {
        for x2 in [pt.x.floor() as i32, (pt.x + 1.0).floor() as i32].iter().cloned() {
        for y2 in [pt.y.floor() as i32, (pt.y + 1.0).floor() as i32].iter().cloned() {
            let l = (glm::vec2(x1,y1),glm::vec2(x2,y2));
            if self.linesegs.contains(&l) {
                let d = dist_to_line_sqr(pt, 
                                         glm::vec2(l.0.x as _ ,l.0.y as _ ), 
                                         glm::vec2(l.1.x as _ ,l.1.y as _ ));
                if d < dist_sqr {
                    next_dist = dist_sqr;
                    dist_sqr = d;
                    thing = Some(l);
                } else if d < next_dist {
                    next_dist = d;
                }
            }
        }
        }
        }
        }
        thing.map(|t| (t,(dist_sqr,next_dist)))
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
                   r.push(Ref::Track(*a,*b));
               }
        }
        r
    }

    pub fn delete(&mut self, x :Ref) {
        match x {
            Ref::Track(a,b) => { self.linesegs.remove(&(a,b)); },
            Ref::Node(a) => { self.node_data.remove(&a); },
            Ref::Object(p) => { self.objects.remove(&p); },
        }
    }
}





