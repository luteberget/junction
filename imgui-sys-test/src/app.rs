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

    pub fn integrate(&mut self, action: EditorAction) {
        match self.handle_event(action) {
            Ok(()) => {},
            Err(_s) => {},
        }
    }

    pub fn handle_event(&mut self, action :EditorAction) -> Result<(), String> {
        match action {
            EditorAction::Inf(InfrastructureEdit::NewTrack(p1,p2)) => {
                let inf = &mut self.model.inf;
                let i1 = self.new_entity(Entity::Node(p1, Node::BufferStop));
                let i2 = self.new_entity(Entity::Node(p2, Node::BufferStop));
                let t =  self.new_entity(Entity::Track(Track {
                    length: p2-p1,
                    start_node: (i1, Port { dir: Dir::Up, course: None }),
                    end_node:   (i2, Port { dir: Dir::Down, course: None }),
                }));
                Ok(())
            },

            EditorAction::Inf(InfrastructureEdit::InsertNode(t,p,node,l)) => {
                Ok(())
            },

            EditorAction::Inf(InfrastructureEdit::JoinNodes(n1,n2)) => {
                Ok(())
            },

            _ => {
                Err("Unhandled EditorAction!".to_string())
            }
        }

    }

    pub fn new_entity(&mut self, ent :Entity) -> EntityId {
        let id = self.model.inf.entities.len();
        self.model.inf.entities.push(Some(ent));
        id
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

type Pos = f32;

pub enum InfrastructureEdit {
    /// Add a new track stretching from Pos to Pos. The track makes a new component.
    NewTrack(Pos,Pos),
    /// Split a track at Pos, inserting a new node with tracks connected to open ends.
    InsertNode(EntityId, Pos, Node, f32),
    /// Join two two-port nodes.
    JoinNodes(EntityId, EntityId),
}

pub enum EditorAction {
    Inf(InfrastructureEdit),
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
    Node(Pos, Node),
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
    Macro(Option<String>),
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




