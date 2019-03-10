use crate::command_builder::*;
use crate::selection::*;

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
                selection: Selection::None,
                viewport: ((0.0,0.0),10.0),
                //selected_object: None,
                hot_route: None,
                selected_movement: None,
                selected_dispatch: None,
                time: 0.0,
                command_builder: None,
                show_imgui_demo : false,
                want_to_quit: false,
            },
        }
    }

    pub fn context_menu(&self) -> Option<CommandScreen> {
        match self.view.selection {
            Selection::Object(id) => {
                match self.model.inf.get(id) {
                    Some(Entity::Track(_)) => {
                        Some(CommandScreen::Menu(Menu { choices: vec![
                            ('p', format!("select mid pos"), |_| None),
                        ]}))
                    },
                    Some(Entity::Node(_,Node::BufferStop)) => {
                        Some(CommandScreen::Menu(Menu { choices: vec![
                            ('e', format!("extend end"), |_| None),
                        ]}))
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }

    pub fn main_menu(&mut self) {
        fn close(app :&mut App) -> Option<CommandScreen> { None }

       let main_menu = Menu {
           choices: vec![
               ('c', format!("context"), |app| { app.context_menu() }),
               ('l', format!("load"),    close ),
               ('s', format!("save"),    close ),
               ('q', format!("quit"),    |app| { app.view.want_to_quit = true; None } ),
               ('s', format!("selection"), |_| { 
                   Some(CommandScreen::Menu(Menu { choices: vec![
                       ('z', format!("none"),      |app| { app.view.selection = Selection::None; None }),
                       ('o', format!("object"),    |app| { app.view.selection = Selection::None; None }),
                       ('p', format!("pos"),       |app| { app.view.selection = Selection::None; None }),
                       ('r', format!("pos range"), |app| { app.view.selection = Selection::None; None }),
                       ('l', format!("path"),      |app| { app.view.selection = Selection::None; None }),
                       ('a', format!("area"),      |app| { app.view.selection = Selection::None; None }),
                   ]}))
               }),

               ('v', format!("view"), |_| { 
                   Some(CommandScreen::Menu(Menu { choices: vec![
                       ('a', format!("all"),       |app| { None }),
                       ('s', format!("selection"), |app| { None }),
                   ]}))
               }),

               ('o', format!("options"), |_| {
                   Some(CommandScreen::Menu(Menu { choices: vec![
                       ('d', format!("imgui debug window"), |app| { 
                           app.view.show_imgui_demo = !app.view.show_imgui_demo; 
                           None })
                   ]}))
               }),
           ]
       };
        self.view.command_builder = Some(CommandBuilder::new_menu(main_menu));
        if self.model.inf.entities.len() == 0 {
            if let CommandScreen::Menu(Menu { choices }) = self.view.command_builder.as_mut().unwrap().current_screen() {
                choices.push(('a', format!("add track"), |app| {
                    app.integrate(EditorAction::Inf(InfrastructureEdit::NewTrack(0.0,100.0)));
                    None
                }));
            }
        }
    }

    pub fn clicked_object(&mut self, id :EntityId) {
        if let Some(cb) = &mut self.view.command_builder {
        } else {
            // todo check if we are in pos selection mode.
            self.view.selection = Selection::Object(id);
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
                        let (straight_side, branch_side) = match node {
                            Node::Switch(_,side) => (side.other(), side),
                            _ => unimplemented!(),
                        };
                        let new = self.new_entity(Entity::Node(p, node.clone()));
                        let inf = &mut self.model.inf;

                        let t = inf.get_track_mut(t).ok_or("Track ref err".to_string())?;

                        match &node {
                            Node::Switch(Dir::Up, _) => {
                                let old_end = t.end_node;

                                t.end_node = (new, Port { dir: Dir::Down, course: None });

                                let _straight = self.new_entity(Entity::Track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(straight_side) }),
                                    end_node: old_end,
                                }));

                                let branch_end = self.new_entity(Entity::Node(p+l, Node::BufferStop));
                                let branch = self.new_entity(Entity::Track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(branch_side) }),
                                    end_node: (branch_end, Port { dir: Dir::Down, course: None }),
                                }));
                            },
                            Node::Switch(Dir::Down, _) => {
                                let old_start = t.start_node;
                                t.start_node = (new, Port { dir: Dir::Up, course: None });

                                let _straight = self.new_entity(Entity::Track(Track {
                                    start_node: old_start,
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(straight_side) })
                                }));

                                let branch_start = self.new_entity(Entity::Node(p-l, Node::BufferStop));
                                let branch = self.new_entity(Entity::Track(Track {
                                    start_node: (branch_start, Port { dir: Dir::Up, course: None }),
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(branch_side) }),
                                }));
                            },
                            _ => unimplemented!()
                        }
                    },
                    InfrastructureEdit::JoinNodes(n1,n2) => {
                        let inf = &mut self.model.inf;
                        let (_,n1_obj) = inf.get_node(n1).ok_or("Node ref err".to_string())?;
                        let (_,n2_obj) = inf.get_node(n2).ok_or("Node ref err".to_string())?;

                        if n1_obj.num_ports() != 1 || n2_obj.num_ports() != 1 {
                            return Err("Nodes must have 1 port.".to_string());
                        }

                        let mut lo_track = None;
                        let mut hi_track = None;

                        for (i,e) in inf.entities.iter().enumerate() {
                            match e {
                                Some(Entity::Track(Track { start_node, end_node, ..  })) => {
                                    if start_node.0 == n1 { hi_track = Some((i,n1)); }
                                    if start_node.0 == n2 { hi_track = Some((i,n2)); }
                                    if end_node.0 == n1   { lo_track = Some((i,n1)); }
                                    if end_node.0 == n2   { lo_track = Some((i,n2)); }
                                },
                                _ => {},
                            };
                        }

                        match (lo_track,hi_track) {
                            (Some((t1,n1)),Some((t2,n2))) => {
                                let end_node = inf.get_track_mut(t2).unwrap().end_node;
                                let track1 = inf.get_track_mut(t1).unwrap();
                                track1.end_node = end_node;
                                inf.delete(t2);
                                inf.delete(n1);
                                inf.delete(n2);
                            },
                            _ => return Err("Mismatching nodes for joining".to_string())
                        }

                    },
                    InfrastructureEdit::ExtendTrack(node_id, length) => {
                        let inf = &mut self.model.inf;
                        let (node_pos,node_type) = inf.get_node_mut(node_id).ok_or("Node ref err".to_string())?;
                        *node_pos += length;
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
    pub selection :Selection,
    //pub selected_object : Option<usize>,
    pub hot_route :Option<usize>,
    pub selected_movement :Option<usize>,
    pub selected_dispatch :Option<usize>,
    pub time :f32,
    pub command_builder : Option<CommandBuilder>,
    pub show_imgui_demo: bool,
    pub want_to_quit: bool,
}

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
    pub fn delete(&mut self, id :EntityId) {
        match self.entities.get_mut(id) {
            Some(mut x) => *x = None,
            _ => {},
        }
    }

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
    pub fn get_node(&self, id :EntityId) -> Option<(&f32,&Node)> {
        if let Some(Some(Entity::Node(ref p,ref t))) = self.entities.get(id) {
            Some((p,t))
        } else { None }
    }
    pub fn get_node_mut(&mut self, id :EntityId) -> Option<(&mut f32,&mut Node)> {
        if let Some(Some(Entity::Node(ref mut p,ref mut t))) = self.entities.get_mut(id) {
            Some((p,t))
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
        //println!("update_schematic");
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
    /// Extend a track by moving its end node forward. There must be enough 
    /// linear space before/after the node.
    ExtendTrack(EntityId, f32),
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
impl Side {
    pub fn other(&self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

#[derive(Debug,Clone)]
pub enum Node {
    Switch(Dir,Side),
    Crossing,
    BufferStop,
    Macro(Option<String>),
}

impl Node {
    pub fn num_ports(&self) -> usize {
        match self {
            Node::Switch (_,_) => 3,
            Node::Crossing => 4,
            Node::BufferStop | Node::Macro(_) => 1,
        }
    }
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




