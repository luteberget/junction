pub struct App {
    pub model : Model,
    pub view : View,
}

impl App {
    pub fn new() -> App {
        App {
            model: Model {
                inf: Infrastructure {
                    entities: vec![],
                    schematic: Derive::Wait,
                },
                routes: vec![],
                scenarios: vec![],
                errors: vec![],
            },
            view: View {
                viewport: ((0.0,0.0),10.0),
                selected_object: None,
                hot_route: None,
                selected_movement: None,
                selected_dispatch: None,
                time: 0.0,
                command_builder: None,
            },
        }
    }
}

pub struct View {
    pub viewport : ((f64,f64),f64),
    pub selected_object : Option<usize>,
    pub hot_route :Option<usize>,
    pub selected_movement :Option<usize>,
    pub selected_dispatch :Option<usize>,
    pub time :f32,
    pub command_builder : Option<CommandBuilder>,
}

pub struct CommandBuilder {}

pub struct Model {
    pub inf :Infrastructure,
    pub routes :Vec<Route>,
    pub scenarios :Vec<Movement>,
    pub errors: Vec<Error>,
}

pub enum Derive<T> {
    Wait,
    Ok(T),
    Error(String), // TODO CString?
}

pub struct Infrastructure {
    pub entities :Vec<Option<Entity>>,
    pub schematic :Derive<Schematic>,
}

use std::collections::HashMap;
pub type EntityId = usize;
pub type Map<K,V> = HashMap<K,V>;

pub struct Port {
    dir: Dir, // Up = pointing outwards from the node, Down = inwards
    course: Option<Side>, // None = trunk/begin/end, Some(Left) = Left switch/crossing
}

pub enum Entity {
    Track(Track),
    Node(Node),
    Object(Object),
}

pub enum Object {
    Signal(Dir),
    Balise(bool),
}

pub struct Track {
    length: f32,
    start_node: (EntityId,Port),
    end_node: (EntityId,Port),
}

pub enum Dir { Up, Down }
pub enum Side { Left, Right }

pub enum Node {
    Switch(Dir,Side),
    Crossing,
    BufferStop,
    Macro(usize),
}


pub type Pt = (f32,f32);
pub type PLine = Vec<Pt>;

pub struct Schematic {
    lines :Map<EntityId, PLine>,
    points: Map<EntityId, Pt>,
}

pub struct Route {
    pub start :usize,
    pub end :usize,
}

pub struct Movement {
    pub spec : (),
    pub dispatches: Vec<Dispatch>,
}

pub struct Dispatch {
}

pub struct Error {
}




