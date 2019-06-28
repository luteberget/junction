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

