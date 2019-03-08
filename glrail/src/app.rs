

pub struct App {
    pub model : Model,
    pub view : View,
}

impl App {
    pub fn new() -> App {
        App {
            model: Model {
                inf: Infrastructure::new_empty(),
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

    pub fn update(&mut self) {
        self.model.inf.check_updates();
    }

    pub fn integrate(&mut self, action: EditorAction) {
        match self.handle_event(action) {
            Ok(()) => {},
            Err(_s) => {},
        }
    }

    pub fn handle_event(&mut self, action :EditorAction) -> Result<(), String> {
        match action {
            EditorAction::Inf(ie) => {
                match ie {
                    InfrastructureEdit::NewTrack(p1,p2) => {
                        let inf = &mut self.model.inf;
                        let i1 = self.new_entity(Entity::Node(p1, Node::BufferStop));
                        let i2 = self.new_entity(Entity::Node(p2, Node::BufferStop));
                        let t =  self.new_entity(Entity::Track(Track {
                            start_node: (i1, Port { dir: Dir::Up, course: None }),
                            end_node:   (i2, Port { dir: Dir::Down, course: None }),
                        }));
                    }
                    InfrastructureEdit::InsertNode(t,p,node,l) => {
                        let new = self.new_entity(Entity::Node(p, node));
                        let inf = &mut self.model.inf;
                        let t = inf.get_track_mut(t).ok_or("Track ref err".to_string())?;
                        let end = t.end_node;
                        t.end_node = (new, Port { dir: Dir::Down, course: None });
                        let trunk = self.new_entity(Entity::Track(Track {
                            start_node: (new, Port { dir: Dir::Up, course: Some(Side::Right) }),
                            end_node: end,
                        }));
                        let branch_end = self.new_entity(Entity::Node(p+l, Node::BufferStop));
                        let branch = self.new_entity(Entity::Track(Track {
                            start_node: (new, Port { dir: Dir::Up, course: Some(Side::Left) }),
                            end_node: (branch_end, Port { dir: Dir::Down, course: None }),
                        }));
                        println!("Inserted node {:?}", self.model.inf.entities);
                    },
                    InfrastructureEdit::JoinNodes(n1,n2) => {
                    },
                };
                // infrastructure changed, update schematic
                self.model.inf.update_schematic();
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

use std::sync::mpsc;
pub struct Infrastructure {
    pub entities :Vec<Option<Entity>>,
    pub schematic :Derive<Schematic>,
    jobs: mpsc::Sender<Vec<Option<Entity>>>,
    results: mpsc::Receiver<Result<Schematic,String>>,
}

impl Infrastructure {
    pub fn get(&self, id :EntityId) -> Option<&Entity> {
        self.entities.get(id)?.as_ref()
    }
    pub fn get_track(&self, id :EntityId) -> Option<&Track> {
        if let Some(Some(Entity::Track(ref t))) = self.entities.get(id) {
            Some(t)
        } else { None }
    }
    pub fn get_track_mut(&mut self, id :EntityId) -> Option<&mut Track> {
        if let Some(Some(Entity::Track(ref mut t))) = self.entities.get_mut(id) {
            Some(t)
        } else { None }
    }
    pub fn get_node(&self, id :EntityId) -> Option<(f32,&Node)> {
        if let Some(Some(Entity::Node(p,ref t))) = self.entities.get(id) {
            Some((*p,t))
        } else { None }
    }

    pub fn new_empty() -> Self {
        use std::thread;

        let (jobs_tx, jobs_rx) = mpsc::channel();
        let (results_tx, results_rx) = mpsc::channel();

        thread::spawn(move || {
            use crate::schematic;
            while let Ok(job) = jobs_rx.recv() {
                // ...
                //
                //
                let r = schematic::solve(&job);
                results_tx.send(r).unwrap();
                super::wake();
            }
            // Exit when channel is closed.
        });

        Infrastructure {
            entities: vec![],
            schematic: Derive::Ok(Schematic { lines: HashMap::new(), points: HashMap::new() }),
            jobs: jobs_tx,
            results: results_rx,
        }
    }
    
    pub fn check_updates(&mut self) {
        while let Ok(s) = self.results.try_recv() {
            match s {
                Ok(s) => self.schematic = Derive::Ok(s),
                Err(s) => self.schematic = Derive::Error(s),
            };
        }
    }

    pub fn update_schematic(&mut self) {
        println!("update_schematic");
        self.schematic = Derive::Wait;
        self.jobs.send(self.entities.clone()).unwrap();
    }
}

pub type Pos = f32;

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


#[derive(Debug,Clone)]
pub enum Entity {
    Track(Track),
    Node(Pos, Node),
    Object(Object),
}

#[derive(Debug,Clone)]
pub enum Object {
    Signal(Dir),
    Balise(bool),
}

#[derive(Debug,Clone)]
pub struct Track {
    pub start_node: (EntityId,Port),
    pub end_node: (EntityId,Port),
}

#[derive(Debug,Clone,Copy)]
pub struct Port {
    pub dir: Dir, // Up = pointing outwards from the node, Down = inwards
    pub course: Option<Side>, // None = trunk/begin/end, Some(Left) = Left switch/crossing
}
#[derive(Debug,Clone,Copy)]
pub enum Dir { Up, Down }
#[derive(Debug,Clone,Copy)]
pub enum Side { Left, Right }

#[derive(Debug,Clone)]
pub enum Node {
    Switch(Dir,Side),
    Crossing,
    BufferStop,
    Macro(Option<String>),
}


pub type Pt = (f32,f32);
pub type PLine = Vec<Pt>;

pub struct Schematic {
    pub lines :Map<EntityId, PLine>,
    pub points: Map<EntityId, Pt>,
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




