use crate::command_builder::*;
use crate::selection::*;
use ordered_float::OrderedFloat;

pub enum InputDir {
    Up, Down, Left, Right
}

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

    pub fn move_view(&mut self, inputdir: InputDir) {
        match inputdir {
            InputDir::Left => (self.view.viewport.0).0 -= 0.15*self.view.viewport.1,
            InputDir::Right => (self.view.viewport.0).0 += 0.15*self.view.viewport.1,
            InputDir::Up => (self.view.viewport.0).1 += 0.15*self.view.viewport.1,
            InputDir::Down => (self.view.viewport.0).1 -= 0.15*self.view.viewport.1,
        }
    }

    pub fn include_in_view(&mut self, pt: (f32,f32))  {
        unimplemented!()
    }
    pub fn entity_location(&self, obj :EntityId) -> (f32,f32) {
        unimplemented!()
    }

    pub fn move_selection(&mut self, inputdir: InputDir) {
        println!("move selection");
        match &self.view.selection {
            Selection::None => { 
                if let Some(id) = self.model.inf.any_object() {
                    self.view.selection = Selection::Object(id);
                    self.include_in_view(self.entity_location(id));
                }
        println!("move selection: none");
            },
            Selection::Object(i) => {
                //if let Some(Some(Entity::Node(_, n))) = self.model.inf.entities.get(*i) {
                //    for p in app.model.inf.node_ports(i) {
                //        match (n,p) {
                //            (Node::BufferStop, Port::Out) => {
                //                // ...
                //            },
                //        }
                //    }
                //}
            },
            Selection::Pos(pos, y, track_id) => {
        println!("move selection: pos");
                if let Some(Some(Entity::Track(Track { start_node, end_node, ..}))) = self.model.inf.entities.get(*track_id) {
                    match inputdir {
                        InputDir::Right => { 
                            self.view.selection = Selection::Object(end_node.0);
                            self.include_in_view(self.entity_location(end_node.0));
                        },
                        InputDir::Left => { 
                            self.view.selection = Selection::Object(start_node.0);
                            self.include_in_view(self.entity_location(start_node.0));
                        },
                        _ => {},
                    }
                }
            },
            _ => { unimplemented!() },
        }
    }

    pub fn middle_of_track(&self, obj :Option<EntityId>) -> Option<(EntityId, f32)> {
        let id = obj?;
        let Track { ref start_node, ref end_node, .. } = self.model.inf.get_track(id)?;
        let (p1,_) = self.model.inf.get_node(start_node.0)?;
        let (p2,_) = self.model.inf.get_node(end_node.0)?;
        Some((id, 0.5*(p1+p2)))
    }


    pub fn context_menu(&self) -> Option<CommandScreen> {
        match self.view.selection {
            Selection::Object(id) => {
                match self.model.inf.get(id) {
                    Some(Entity::Track(_)) => {
                        Some(CommandScreen::Menu(Menu { choices: vec![
                            ('p', format!("select mid pos"), |app| {
                                if let Selection::Object(id) = &app.view.selection { 
                                    if let Some(Track { start_node, end_node, .. }) = app.model.inf.get_track(*id) {
                                        let (n1_pos,_) = app.model.inf.get_node(start_node.0).unwrap();
                                        let (n2_pos,_) = app.model.inf.get_node(end_node.0).unwrap();
                                        app.select_pos(0.5*(n1_pos + n2_pos), *id);
                                    }
                                }
                                None
                            }),
                        ]}))
                    },
                    Some(Entity::Node(_,Node::BufferStop)) | Some(Entity::Node(_, Node::Macro(_))) => {
                        Some(CommandScreen::Menu(Menu { choices: vec![
                            ('e', format!("extend end"), |app| {
                                let mut arguments = ArgumentListBuilder::new();
                                if let Selection::Object(id) = &app.view.selection {
                                    arguments.add_id_value("node", *id);
                                } else {
                                    arguments.add_id("node");
                                }
                                arguments.add_float_default("length", 50.0);
                                arguments.set_action(|app :&mut App,args :&ArgumentListBuilder| {
                                    let id = *args.get_id("node").unwrap();
                                    let l  = *args.get_float("length").unwrap();
                                    app.integrate(EditorAction::Inf(
                                            InfrastructureEdit::ExtendTrack(id, l)));
                                });
                                Some(CommandScreen::ArgumentList(arguments))
                            }),
                            ('j', format!("join with node"), |app| {
                                let mut arguments = ArgumentListBuilder::new();
                                if let Selection::Object(id) = &app.view.selection {
                                    arguments.add_id_value("node1",*id);
                                } else {
                                    arguments.add_id("node1");
                                }
                                arguments.add_id("node2");
                                arguments.set_action(|app :&mut App, args :&ArgumentListBuilder| {
                                    let n1 = *args.get_id("node1").unwrap();
                                    let n2 = *args.get_id("node2").unwrap();
                                    app.integrate(EditorAction::Inf(
                                            InfrastructureEdit::JoinNodes(n1, n2)));
                                });
                                Some(CommandScreen::ArgumentList(arguments))
                            }),
                        ]}))
                    },
                    _ => None,
                }
            },
            Selection::Pos(pos,y,id) => {
                Some(CommandScreen::Menu(Menu { choices: vec![
                    ('k', format!("out left sw"), |app| { 
                        if let Selection::Pos(pos,_,track_id) = &app.view.selection {
                        app.integrate(EditorAction::Inf(
                            InfrastructureEdit::InsertNode(
                            *track_id, *pos, Node::Switch(Dir::Up, Side::Left), 50.0)));
                        }
                        None }),
                    ('K', format!("in right sw"), |app| { 
                        if let Selection::Pos(pos,_,track_id) = &app.view.selection {
                        app.integrate(EditorAction::Inf(
                            InfrastructureEdit::InsertNode(
                            *track_id, *pos, Node::Switch(Dir::Down, Side::Right), 50.0)));
                        }
                        None }),
                    ('j', format!("out right sw"), |app| { 
                        if let Selection::Pos(pos,_,track_id) = &app.view.selection {
                        app.integrate(EditorAction::Inf(
                            InfrastructureEdit::InsertNode(
                            *track_id, *pos, Node::Switch(Dir::Up, Side::Right), 50.0)));
                        }
                        None }),
                    ('J', format!("in left sw"), |app| { 
                        if let Selection::Pos(pos,_,track_id) = &app.view.selection {
                        app.integrate(EditorAction::Inf(
                            InfrastructureEdit::InsertNode(
                            *track_id, *pos, Node::Switch(Dir::Down, Side::Left), 50.0)));
                        }
                        None }),
                ]}))
            }
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
                       ('o', format!("object"),    |app| { 
                           if let Some(id) = app.model.inf.any_object() {
                               app.view.selection = Selection::Object(id);
                           }
                           None 
                       }),
                       ('p', format!("pos"),       |app| { 
                           app.view.selection = Selection::None; 
                           None 
                       }),
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

    pub fn clicked_object(&mut self, id :Option<EntityId>, location: (f32,f32)) {
        println!("Clicked {:?} {:?}", id, location);
        if let Some(id) = id {
            if let Some(cb) = &mut self.view.command_builder {
                if let CommandScreen::ArgumentList(ref mut alb) = cb.current_screen() {
                    for (n,s,a) in &mut alb.arguments {
                        if let Arg::Id(ref mut optid) = a {
                            if let ArgStatus::NotDone = s {
                                *optid = Some(id);
                                break;
                            }
                        }
                    }
                }
            } else {
                // todo check if we are in pos selection mode.

                if let Some(Some(Entity::Track(_))) = self.model.inf.entities.get(id) {
                    if let Derive::Ok(ref s) = &self.model.inf.schematic {
                        if let Some(pos) = s.x_to_pos(location.0) {
                            self.view.selection = Selection::Pos(pos, 0.0, id);
                        }
                    }
                } else { 
                    self.view.selection = Selection::Object(id);
                }
            }
        }
    }

    pub fn select_pos(&mut self, pos :f32, obj :EntityId) {
        let y = 0.0;
        self.view.selection = Selection::Pos(pos, y, obj );
        //println!("select pos {:?}", self.view.selection);
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
                        };

                        self.view.selection = Selection::Object(new);

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
    pub fn any_object(&self) -> Option<EntityId> {
        for (i,x) in self.entities.iter().enumerate() {
            if x.is_some() { 
                return Some(i);
            }
        }
        None
    }

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
            schematic: Derive::Ok(Schematic { lines: HashMap::new(), points: HashMap::new(), pos_map: vec![] }),
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
    pub pos_map: Vec<(f32, EntityId, f32)>,
}

impl Schematic {
    pub fn x_to_pos(&self, x: f32) -> Option<f32> {
        match self.pos_map.binary_search_by_key(&OrderedFloat(x), |&(x,_,p)| OrderedFloat(x)) {
            Ok(i) => {
                Some(self.pos_map[i].0)
            },
            Err(i) => {
                if i <= 0 || i >= self.pos_map.len() {
                    return None;
                }
                let prev = self.pos_map[i-1];
                let next = self.pos_map[i];
                //
                // lerp prev->next by x
                Some(prev.2 + (next.2-prev.2)*(x - prev.0)/(next.0 - prev.0))
            }
        }
    }

    pub fn find_pos(&self, pos :f32) -> Option<f32> {
        match self.pos_map.binary_search_by_key(&OrderedFloat(pos), |&(x,_,p)| OrderedFloat(p)) {
            Ok(i) => Some(self.pos_map[i].2),
            Err(i) => {
                if i <= 0 || i >= self.pos_map.len() {
                    return None;
                }
                let prev = self.pos_map[i-1];
                let next = self.pos_map[i];

                // lerp prev->next by pos
                Some(prev.0 + (next.0-prev.0)*(pos - prev.2)/(next.2 - prev.2))
            },
        }
    }
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




