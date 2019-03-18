pub use railml2dgraph::dgraph::DGraphModel as DGraph;
pub use railml2dgraph::routes::ConvertRouteIssue;
pub use railml2dgraph::dgraph::*;
use crate::infrastructure::*;
use crate::interlocking::*;
use std::collections::HashMap;
use ordered_float::OrderedFloat;

pub fn make_routes(dgraph :&DGraph) -> (Vec<Route>,Vec<ConvertRouteIssue>) {
    railml2dgraph::routes::convert_routes(dgraph)
}

pub fn convert_entities(ent :&Vec<Option<Entity>>) -> Result<DGraph,String> {
    // Convert entitiy graph to dgraph

    //println!("CONVERTING {:?}", ent);
    let mut tracks = HashMap::new();

    struct OTrack<'a> {
        pos_start :f32,
        pos_end :f32,
        start :(&'a Node, (usize, Port)),
        end :(&'a Node, (usize, Port)),
        objs :Vec<(f32, &'a Object)>,
    }

    struct DSwitch {
        trunk :Option<PartNodeIdx>,
        left :Option<PartNodeIdx>,
        right :Option<PartNodeIdx>,
    }

    for (i,e) in ent.iter().enumerate() {
        if let Some(Entity::Track(Track { ref start_node, ref end_node })) = e {
            let (p1,n1) = if let Some(Some(Entity::Node(p,n))) = ent.get(start_node.0) { Ok((p,n)) } 
                     else { Err("Invalid node ref.".to_string()) } ?;
            let (p2,n2) = if let Some(Some(Entity::Node(p,n))) = ent.get(end_node.0) { Ok((p,n)) } 
                     else { Err("Invalid node ref.".to_string()) } ?;

            tracks.insert(i, OTrack {
                pos_start: *p1, pos_end: *p2, start: (n1,*start_node), end: (n2,*end_node), objs: Vec::new()
            });
        }
    }

    for (i,e) in ent.iter().enumerate() {
        if let Some(Entity::Object(t,p,obj)) = e {
            let track = tracks.get_mut(t).ok_or("Invalid track ref.".to_string())?;
            track.objs.push((*p,obj));
        }
    }

    let mut dswitches = HashMap::new();
    for (i,e) in ent.iter().enumerate() {
        if let Some(Entity::Node(p,Node::Switch(_,_))) = e {
            dswitches.insert(i, DSwitch { trunk: None, left: None, right: None });
        }
    }

    //let mut named_connections = HashMap::new();
    let mut dgraph = DGraph::default();

    let mut unnamed_boundaries = 0;
    for (track_idx, mut t) in tracks {
        let (mut na, mut nb) = new_node(&mut dgraph.nodes);
        match t.start {
            (Node::BufferStop, _) => {},
            (Node::Macro(name), _) => {
                dgraph.edges.push(Edge::Boundary(na));
                dgraph.nodes[nb.node_idx()].a.name = name.as_ref().cloned().unwrap_or_else(|| { 
                    unnamed_boundaries += 1; 
                    format!("b{}", unnamed_boundaries) });
            },
            (Node::Switch(Dir::Down,side), (sw_idx, Port { dir: Dir::Up, ..}))  => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(na);
            },
            (Node::Switch(Dir::Up,side), (sw_idx, Port { dir: Dir::Up, course: Some(c) }))  => {
                match c {
                    Side::Left => {  dswitches.get_mut(&sw_idx).unwrap().left = Some(na); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(na); },
                }
            },
            (Node::Switch(_,_), _) => panic!("inconsistent switch connections. "),
            _ => unimplemented!(),
        }
        t.objs.sort_by_key(|o| OrderedFloat(o.0));

        let mut last_node = nb;
        let mut last_pos = t.pos_start;

        for (pos,obj) in t.objs {
            let (mut na, mut nb) = new_node(&mut dgraph.nodes);

            match obj {
                Object::Detector => {
                    dgraph.nodes[na.node_idx()].has_detector = true;
                },
                Object::Signal(dir) => {
                    match dir {
                        Dir::Up => 
                            dgraph.nodes[nb.node_idx()].b.objs.push(PartNodeObject::Signal("s1".to_string())),
                        Dir::Down => 
                            dgraph.nodes[na.node_idx()].a.objs.push(PartNodeObject::Signal("s1".to_string())),
                    }
                },
                Object::Balise(_) => {}, // No use for this as of now.
            }

            dgraph.edges.push(Edge::Linear(last_node, (na, (pos - last_pos) as _ )));
            last_pos = pos;
            last_node = nb;
        }

        let (mut na, mut nb) = new_node(&mut dgraph.nodes);
        dgraph.edges.push(Edge::Linear(last_node, (na, (t.pos_end - last_pos) as _ )));

        match t.end {
            (Node::BufferStop, _) => {},
            (Node::Macro(name), _) => {
                dgraph.edges.push(Edge::Boundary(nb));
                dgraph.nodes[nb.node_idx()].b.name = name.as_ref().cloned().unwrap_or_else(|| {
                    unnamed_boundaries += 1;
                    format!("b{}", unnamed_boundaries) });
            },
            (Node::Switch(Dir::Up, _), (sw_idx, Port { dir: Dir::Down, ..})) => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(nb);
            },
            (Node::Switch(Dir::Down, _), (sw_idx, Port { dir: Dir::Down, course: Some(c) })) => {
                match c {
                    Side::Left  => { dswitches.get_mut(&sw_idx).unwrap().left  = Some(nb); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(nb); },
                }
            },
            (Node::Switch(_,_), _) => panic!("Inconsistent switch"),
            _ => unimplemented!(),

        }
    }

    // Resolve switches
    for (i,s) in dswitches {
        let trunk = s.trunk.ok_or(format!("Inconsistent switch data."))?;
        let left = s.left.ok_or(format!("Inconsistent switch data."))?;
        let right = s.right.ok_or(format!("Inconsistent switch data."))?;

        dgraph.edges.push(Edge::Switch(format!("sw{}", i), None,
                trunk, (left, 0.0), (right, 0.0)));
    }


    Ok(dgraph)

}








