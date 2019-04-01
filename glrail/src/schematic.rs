use crate::app::*;
use std::collections::HashMap;
use ordered_float::OrderedFloat;
use std::sync::mpsc;
use railplotlib;
use crate::model::*;
use crate::infrastructure::*;
use serde::{Serialize, Deserialize};

pub type Pt = (f32,f32);
pub type PLine = Vec<Pt>;
pub type Map<K,V> = HashMap<K,V>;

#[derive(Serialize, Deserialize)]
pub struct Schematic {
    pub lines :Map<TrackId, PLine>,
    pub points: Map<NodeId, Pt>,
    pub pos_map: Vec<(f32, NodeId, f32)>,
}


fn lerp(v0 :f32, v1 :f32, t: f32) -> f32 {
    (1.0-t)*v0 + t*v1
}

impl Schematic {
    pub fn new_empty() -> Self {
        Schematic {
            lines: Map::new(),
            points: Map::new(),
            pos_map: Vec::new(),
        }
    }
    pub fn track_line_at(&self, track :&TrackId, pos :f32) -> Option<((f32,f32),(f32,f32))> {
        let x = self.find_pos(pos)?;
        let line = self.lines.get(track)?;
        for ((x0,y0),(x1,y1)) in line.iter().zip(line.iter().skip(1)) {
            if *x0 <= x && x <= *x1 {
                let y = lerp(*y0,*y1, (x-*x0)/(*x1-*x0));
                let pt = (x,y);
                let tangent = (*x1-*x0,*y1-*y0);
                let len = ((*x1-*x0)*(*x1-*x0)+(*y1-*y0)*(*y1-*y0)).sqrt();
                let tangent = (tangent.0 / len, tangent.1 / len);
                return Some((pt,tangent));
            }
        }
        None
    }
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
                    if i == 0 && (self.pos_map[0].2 - pos).abs() < 1e-6 {
                        return Some(self.pos_map[0].0);
                    }
                    if i == self.pos_map.len()-1 && (self.pos_map[self.pos_map.len()-1].2 - pos).abs() < 1e-6 {
                        return Some(self.pos_map[self.pos_map.len()].0);
                    }
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


// This file should encapsulate railplotlib, it is to be
// the only interface to railplotlib in glrail.

fn conv_side(side :Side) -> railplotlib::model::Side {
    match side {
        Side::Left => railplotlib::model::Side::Left,
        Side::Right => railplotlib::model::Side::Right,
    }
}

fn conv_dir(dir :Dir) -> railplotlib::model::Dir {
    match dir {
        Dir::Up => railplotlib::model::Dir::Up,
        Dir::Down => railplotlib::model::Dir::Down,
    }
}

// fn conv_side_inv(side :railplotlib::model::Side) -> Side {
//     match side {
//         railplotlib::model::Side::Left => Side::Left,
//         railplotlib::model::Side::Right => Side::Right,
//     }
// }
// 
// fn conv_dir_inv(dir :railplotlib::model::Dir) -> Dir {
//     match dir {
//         railplotlib::model::Dir::Up   => Dir::Up,
//         railplotlib::model::Dir::Down => Dir::Down,
//     }
// }

fn conv_port(port :Port, node :&NodeType) -> railplotlib::model::Port {
    match node {
        NodeType::BufferStop | NodeType::Macro(_) => {
            match port.dir {
                Dir::Up => railplotlib::model::Port::Out,
                Dir::Down => railplotlib::model::Port::In,
            }
        },
        NodeType::Switch(dir, _) => {
            match (port.dir, dir) {
                (Dir::Down, Dir::Up)     => railplotlib::model::Port::Trunk,
                (Dir::Up, Dir::Down) => railplotlib::model::Port::Trunk,
                _ => match port.course {
                    Some(Side::Left) => railplotlib::model::Port::Left,
                    Some(Side::Right) => railplotlib::model::Port::Right,
                    _ => panic!(),
                },
            }
        },
        NodeType::Crossing => {
            unimplemented!()
        },
    }
}

pub fn solve(inf :&Infrastructure) -> Result<Schematic, String> {

    // convert from
    //     1: track from 3.trunk to 5.left
    //     2: endnode
    //     4: swithc
    //     ...
    // into
    //     node x asf. ..
    //     edge a.trunk b.left

    let mut nodes = Vec::new();
    let mut edges :Vec<railplotlib::model::Edge<()>> = Vec::new();

    let mut buffer_side = HashMap::new();
    for (_id,Track { start_node, end_node, .. }) in inf.iter_tracks() {
        if let Some(Node(_, NodeType::BufferStop)) = inf.get_node(&start_node.0) {
            buffer_side.insert(start_node.0, railplotlib::model::Shape::Begin);
        }
        if let Some(Node(_, NodeType::BufferStop)) = inf.get_node(&&end_node.0) {
            buffer_side.insert(end_node.0, railplotlib::model::Shape::End);
        }
        if let Some(Node(_, NodeType::Macro(_))) = inf.get_node(&start_node.0) {
            buffer_side.insert(start_node.0, railplotlib::model::Shape::Begin);
        }
        if let Some(Node(_, NodeType::Macro(_))) = inf.get_node(&end_node.0) {
            buffer_side.insert(end_node.0, railplotlib::model::Shape::End);
        }
    }

    let mut track_idxs = Vec::new();
    let mut node_idxs = Vec::new();

    for (track_id, Track { start_node, end_node, .. }) in inf.iter_tracks() {
        let Node(_,ref n1) = inf.get_node(&start_node.0).unwrap();
        let Node(_,ref n2) = inf.get_node(&end_node.0).unwrap();
        edges.push(railplotlib::model::Edge {
            objects: Vec::new(),
            a: (format!("n{:?}", start_node.0), conv_port(start_node.1, n1)),
            b: (format!("n{:?}", end_node.0),   conv_port(end_node.1,   n2)),
        });
        track_idxs.push(track_id);
    }

    for (node_id, Node(pos, node)) in inf.iter_nodes() {
        match node {
            NodeType::Switch(dir,side) => {
                nodes.push(railplotlib::model::Node {
                    name: format!("n{:?}", node_id),
                    pos: *pos as _,
                    shape: railplotlib::model::Shape::Switch(conv_side(*side),conv_dir(*dir)),
                });
                node_idxs.push(node_id);
            },
            NodeType::Crossing => unimplemented!(),
            NodeType::BufferStop | NodeType::Macro(_) => {
                nodes.push(railplotlib::model::Node {
                    name: format!("n{:?}", node_id),
                    pos: *pos as _,
                    shape: buffer_side[&node_id],
                });
                node_idxs.push(node_id);
            }
        }
    }

    // TODO objects -> symbols

    let model = railplotlib::model::SchematicGraph { nodes, edges };
    //println!("Model: {:#?}", model);

    let sol = {
        use railplotlib::solvers::SchematicSolver;
        use railplotlib::solvers::Goal;
        let solver = railplotlib::solvers::LevelsSatSolver {
            criteria: vec![
                Goal::Height, 
                Goal::Bends, 
                Goal::Width,
            ],
            nodes_distinct: true,
        };
        solver.solve(model)
    }?;

    //println!("{:?}", sol.nodes);
    //println!("{:#?}", sol.lines);

    //println!("\n\n\n");

    // now, convert it back.
    // j

    let mut output = Schematic {
        lines: HashMap::new(),
        points: HashMap::new(),
        pos_map: Vec::new(),
    };

    for (ni,(n,pt)) in sol.nodes.iter().enumerate() {
        let id = node_idxs[ni];
        output.points.insert(id, (pt.0 as _, pt.1 as _));
        output.pos_map.push((pt.0 as _ /* x value */, id, n.pos as _ ));
    }

    output.pos_map.sort_by_key(|x| OrderedFloat(x.0));

    for (ei,(e,pline)) in sol.lines.iter().enumerate() {
        let id = track_idxs[ei];
        output.lines.insert(id, pline.iter().map(|x| (x.0 as _, x.1 as _)).collect());
    }

    Ok(output)
}
