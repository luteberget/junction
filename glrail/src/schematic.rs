use crate::app::{Entity, Track, Node, Object, Pos, Dir, Side,Schematic, Port};
use std::collections::HashMap;

// This file should encapsulate railplotlib, it is to be
// the only interface to railplotlib in glrail.


use railplotlib;


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

fn conv_port(port :Port, node :&Node) -> railplotlib::model::Port {
    match node {
        Node::BufferStop | Node::Macro(_) => {
            match port.dir {
                Dir::Up => railplotlib::model::Port::Out,
                Dir::Down => railplotlib::model::Port::In,
            }
        },
        Node::Switch(dir, _) => {
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
        Node::Crossing => {
            unimplemented!()
        },
    }
}

pub fn solve(model :&Vec<Option<Entity>>) -> Result<Schematic, String> {

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
    for (i,e) in model.iter().enumerate() {
        if let Some(e) = e {
            match e {
                Entity::Track(Track { start_node, end_node }) =>  {
                    if let Some(Entity::Node(_, Node::BufferStop)) = model[start_node.0] {
                        buffer_side.insert(start_node.0, railplotlib::model::Shape::Begin);
                    }
                    if let Some(Entity::Node(_, Node::BufferStop)) = model[end_node.0] {
                        buffer_side.insert(end_node.0, railplotlib::model::Shape::End);
                    }
                    if let Some(Entity::Node(_, Node::Macro(_))) = model[start_node.0] {
                        buffer_side.insert(start_node.0, railplotlib::model::Shape::Begin);
                    }
                    if let Some(Entity::Node(_, Node::Macro(_))) = model[end_node.0] {
                        buffer_side.insert(end_node.0, railplotlib::model::Shape::End);
                    }
                },
                _ => {},
            }
        }
    }

    let mut track_idxs = Vec::new();
    let mut node_idxs = Vec::new();
    for (i,e) in model.iter().enumerate() {
        if let Some(e) = e {
            match e {
                Entity::Track(Track { start_node, end_node }) => {
                    let n1 = if let Some(Entity::Node(_, ref n)) = &model[start_node.0] { n } else { panic!() };
                    let n2 = if let Some(Entity::Node(_, ref n)) = &model[end_node.0]   { n } else { panic!() };
                    edges.push(railplotlib::model::Edge {
                        objects: Vec::new(),
                        a: (format!("n{}", start_node.0), conv_port(start_node.1, n1)),
                        b: (format!("n{}", end_node.0),   conv_port(end_node.1,   n2)),
                    });
                    track_idxs.push(i);
                },
                Entity::Node(pos, Node::Switch(dir, side)) => {
                    nodes.push(railplotlib::model::Node {
                        name: format!("n{}", i),
                        pos: *pos as _,
                        shape: railplotlib::model::Shape::Switch(conv_side(*side),conv_dir(*dir)),
                    });
                    node_idxs.push(i);
                },
                Entity::Node(pos, Node::Crossing) => {
                    unimplemented!();
                },
                Entity::Node(pos, Node::BufferStop) | Entity::Node(pos, Node::Macro(_)) => {
                    nodes.push(railplotlib::model::Node {
                        name: format!("n{}", i),
                        pos: *pos as _,
                        shape: buffer_side[&i],
                    });
                    node_idxs.push(i);
                },
                Entity::Object(_) => {
                    // ignore for now...
                },
            };
        }
    }

    let model = railplotlib::model::SchematicGraph { nodes, edges };
    println!("Model: {:#?}", model);

    let sol = {
        use railplotlib::solvers::SchematicSolver;
        use railplotlib::solvers::Goal;
        let solver = railplotlib::solvers::LevelsSatSolver {
            criteria: vec![Goal::Bends, Goal::Height, Goal::Width],
            nodes_distinct: true,
        };
        solver.solve(model)
    }?;

    println!("{:?}", sol.nodes);
    println!("{:#?}", sol.lines);

    println!("\n\n\n");

    // now, convert it back.
    // j

    let mut output = Schematic {
        lines: HashMap::new(),
        points: HashMap::new(),
    };

    for (ni,(n,pt)) in sol.nodes.iter().enumerate() {
        let id = node_idxs[ni];
        output.points.insert(id, (pt.0 as _, pt.1 as _));
    }

    for (ei,(e,pline)) in sol.lines.iter().enumerate() {
        let id = track_idxs[ei];
        output.lines.insert(id, pline.iter().map(|x| (x.0 as _, x.1 as _)).collect());
    }

    Ok(output)
}
