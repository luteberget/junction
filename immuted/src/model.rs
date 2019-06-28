use nalgebra_glm as glm;

pub type Pt = glm::I32Vec2;
pub type PtA = glm::I32Vec2;
pub type PtC = glm::Vec2;

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
    pub objects: im::HashMap<PtA, (PtC, Object)>,
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
    pub fn get_closest(&self, pt :PtC) -> Option<Ref> {
        // TODO performance
        let (mut thing,mut dist_sqr) = (None, std::f32::INFINITY);
        for x1 in &[pt.x.floor() as i32, pt.x.ceil() as i32] {
        for y1 in &[pt.y.floor() as i32, pt.y.ceil() as i32] {
        for x2 in &[pt.x.floor() as i32, pt.x.ceil() as i32] {
        for y2 in &[pt.y.floor() as i32, pt.y.ceil() as i32] {
            let l = (glm::vec2(*x1,*y1),glm::vec2(*x2,*y2));
            if self.linesegs.contains(&l) {
                let d = dist_to_line_sqr(pt, 
                                         glm::vec2(l.0.x as _ ,l.0.y as _ ), 
                                         glm::vec2(l.1.x as _ ,l.1.y as _ ));
                if d < dist_sqr {
                    dist_sqr = d;
                    thing = Some(Ref::Track(l.0,l.1));
                }
            }
        }
        }
        }
        }
        thing
    }

    pub fn get_rect(&self, a :PtC, b :PtC) -> Vec<Ref> {
        // TODO performance
        unimplemented!()
    }
}

pub fn project_to_line(p :PtC, a :PtC, b :PtC) -> PtC {
    let t = glm::clamp_scalar(glm::dot(&(p-a),&(b-a)) / glm::distance2(&a,&b), 0.0, 1.0);
    glm::lerp(&a,&b,t)
}

pub fn dist_to_line_sqr(p :PtC, a :PtC, b :PtC) -> f32 {
    glm::length2(&(project_to_line(p,a,b) - p))
}





