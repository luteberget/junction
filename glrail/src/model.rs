use crate::infrastructure::*;
use crate::schematic::*;
use crate::interlocking::*;
use crate::view::*;
use crate::issue::*;
use crate::selection::*;

pub enum Derive<T> {
    Wait,
    Ok(T),
    Err(String),
}

pub enum ModelAction {
    Inf(InfrastructureEdit),
}

pub struct Model {
    pub inf :Infrastructure,
    pub schematic :Derive<Schematic>,
    pub view :View,
    pub interlocking: Interlocking,
}

impl Model {

    pub fn new_empty() -> Self {
        Model {
            inf: Infrastructure::new_empty(),
            schematic: Derive::Ok(Schematic::new_empty()),
            view: View::new_default(),
            interlocking: Interlocking::new_default(),
        }
    }

    pub fn select_pos(&mut self, pos :f32, obj :EntityId) {
        let y = 0.0;
        self.view.selection = Selection::Pos(pos, y, obj );
        //println!("select pos {:?}", self.view.selection);
    }

    pub fn integrate(&mut self, action :ModelAction) {
    }

    pub fn iter_issues(&self) -> impl Iterator<Item = Issue> {
        use std::iter;
        iter::empty()
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
        //unimplemented!()
    }

    pub fn entity_location(&self, obj :EntityId) -> (f32,f32) {
        return (0.0,0.0);
        //unimplemented!()
    }

    pub fn move_selection(&mut self, inputdir: InputDir) {
        println!("move selection");
        match &self.view.selection {
            Selection::None => { 
                if let Some(id) = self.inf.any_object() {
                    self.view.selection = Selection::Entity(id);
                    self.include_in_view(self.entity_location(id));
                }
        println!("move selection: none");
            },
            Selection::Entity(i) => {
                //if let Some(Some(Entity::Node(_, n))) = self.inf.entities.get(*i) {
                //    for p in app.inf.node_ports(i) {
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
                if let Some(Some(Entity::Track(Track { start_node, end_node, ..}))) = self.inf.entities.get(*track_id) {
                    match inputdir {
                        InputDir::Right => { 
                            self.view.selection = Selection::Entity(end_node.0);
                            self.include_in_view(self.entity_location(end_node.0));
                        },
                        InputDir::Left => { 
                            self.view.selection = Selection::Entity(start_node.0);
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
        let Track { ref start_node, ref end_node, .. } = self.inf.get_track(id)?;
        let (p1,_) = self.inf.get_node(start_node.0)?;
        let (p2,_) = self.inf.get_node(end_node.0)?;
        Some((id, 0.5*(p1+p2)))
    }

    pub fn handle_event(&mut self, action :ModelAction) -> Result<(), String> {
        match action {
            ModelAction::Inf(ie) => {
                match ie {
                    InfrastructureEdit::NewTrack(p1,p2) => {
                        let inf = &mut self.inf;
                        let i1 = self.inf.new_entity(Entity::Node(p1, Node::BufferStop));
                        let i2 = self.inf.new_entity(Entity::Node(p2, Node::BufferStop));
                        let t =  self.inf.new_entity(Entity::Track(Track {
                            start_node: (i1, Port { dir: Dir::Up, course: None }),
                            end_node:   (i2, Port { dir: Dir::Down, course: None }),
                        }));
                    },
                    InfrastructureEdit::InsertObject(t,p,obj) => {
                        let _id = self.inf.new_entity(Entity::Object(t,p,obj));
                    },
                    InfrastructureEdit::InsertNode(t,p,node,l) => {
                        let (straight_side, branch_side) = match node {
                            Node::Switch(_,side) => (side.other(), side),
                            _ => unimplemented!(),
                        };
                        let new = self.inf.new_entity(Entity::Node(p, node.clone()));
                        let inf = &mut self.inf;

                        let t = inf.get_track_mut(t).ok_or("Track ref err".to_string())?;

                        match &node {
                            Node::Switch(Dir::Up, _) => {
                                let old_end = t.end_node;

                                t.end_node = (new, Port { dir: Dir::Down, course: None });

                                let _straight = self.inf.new_entity(Entity::Track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(straight_side) }),
                                    end_node: old_end,
                                }));

                                let branch_end = self.inf.new_entity(Entity::Node(p+l, Node::BufferStop));
                                let branch = self.inf.new_entity(Entity::Track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(branch_side) }),
                                    end_node: (branch_end, Port { dir: Dir::Down, course: None }),
                                }));
                            },
                            Node::Switch(Dir::Down, _) => {
                                let old_start = t.start_node;
                                t.start_node = (new, Port { dir: Dir::Up, course: None });

                                let _straight = self.inf.new_entity(Entity::Track(Track {
                                    start_node: old_start,
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(straight_side) })
                                }));

                                let branch_start = self.inf.new_entity(Entity::Node(p-l, Node::BufferStop));
                                let branch = self.inf.new_entity(Entity::Track(Track {
                                    start_node: (branch_start, Port { dir: Dir::Up, course: None }),
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(branch_side) }),
                                }));
                            },
                            _ => unimplemented!()
                        };

                        self.view.selection = Selection::Entity(new);

                    },
                    InfrastructureEdit::JoinNodes(n1,n2) => {
                        let inf = &mut self.inf;
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
                        let inf = &mut self.inf;
                        let (node_pos,node_type) = inf.get_node_mut(node_id).ok_or("Node ref err".to_string())?;
                        *node_pos += length;
                    },
                };
                // infrastructure changed, update schematic
                // TODO self.model.inf.update_schematic();
                Ok(())
            },
            _ => {
                Err("Unhandled ModelAction!".to_string())
            }
        }
    }


}



