use rolling::input::staticinfrastructure as rolling_inf;
use crate::infrastructure::*;
use crate::interlocking::{ Route };
use std::hash::Hash;
use std::collections::{HashMap, HashSet}; 
use ordered_float::OrderedFloat;
use std::sync::Arc;
use bimap::BiMap;

pub use route_finder::ConvertRouteIssue;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RollingId {
    StaticObject(rolling_inf::ObjectId),
    Node(rolling_inf::NodeId),
}


#[derive(Debug)]
pub struct DGraph {
    /// static infrastructure with internal indexing (names do not correspond to glrail entities)
    pub rolling_inf : Arc<rolling_inf::StaticInfrastructure>,
    /// Reference from rolling dgraph indices to glrail entitiy vec
    pub entity_names : BiMap<RollingId, EntityId>,
    /// tvd sections mapped to set of edges
    pub tvd_sections :HashMap<rolling_inf::ObjectId, Vec<(rolling_inf::NodeId,rolling_inf::NodeId)>>,
    /// edges mapped to track interval (track in glrail model)
    pub edge_intervals : HashMap<(rolling_inf::NodeId,rolling_inf::NodeId),Interval>,
    // /// string names of boundaries, to be referred to in dispatch plans
    // pub boundary_names :HashMap<String, rolling_inf::NodeId>,
}

impl Default for DGraph {
    fn default() -> Self {
        DGraph {
            rolling_inf: Arc::new(rolling_inf::StaticInfrastructure { nodes: vec![], objects: vec![] }),
            entity_names: BiMap::new(),
            tvd_sections: HashMap::new(),
            edge_intervals: HashMap::new(),
            //boundary_names: HashMap::new(),
        }
    }
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Interval {
    pub track :TrackId,
    pub p1 :Pos,
    pub p2 :Pos,
}

pub enum DGraphConvertIssue {
    UnnamedBoundary(NodeId),
    MisconfiguredSwitch(NodeId),
}


pub fn convert_entities(inf :&Infrastructure) -> Result<(DGraph,Vec<DGraphConvertIssue>), String>  {
    let mut issues = Vec::new();
    let mut tracks = HashMap::new();
    let mut entity_names = BiMap::new();
    let mut edge_intervals = HashMap::new();
    let mut detector_nodes = HashSet::new();
    //let mut boundary_names = HashMap::new();

    let mut model = rolling_inf::StaticInfrastructure {
        nodes: Vec::new(),
        objects: Vec::new(),
    };

    fn new_pair(nodes :&mut Vec<rolling_inf::Node>) -> (rolling_inf::NodeId,rolling_inf::NodeId) {
        let (i1,i2) = (nodes.len(), nodes.len()+1);
        nodes.push(rolling_inf::Node { other_node: i2, 
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        nodes.push(rolling_inf::Node { other_node: i1, 
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        (i1,i2)
    }

    fn new_object_id(objects :&mut Vec<rolling_inf::StaticObject>, obj :rolling_inf::StaticObject) -> rolling_inf::ObjectId {
        let idx = objects.len();
        objects.push(obj);
        idx
    }

    fn join_linear(nodes :&mut Vec<rolling_inf::Node>, 
                   edge_intervals :&mut HashMap<(rolling_inf::NodeId,rolling_inf::NodeId),Interval>, 
                   interval: Interval, i1 :rolling_inf::NodeId, i2 :rolling_inf::NodeId, dist :f64) {
        nodes[i1].edges = rolling_inf::Edges::Single(i2, dist);
        nodes[i2].edges = rolling_inf::Edges::Single(i1, dist);
        edge_intervals.insert((i1,i2),interval);
        edge_intervals.insert((i2,i1),interval);
    }

    struct OTrack<'a> {
        pos_start :f32,
        pos_end :f32,
        start :(&'a NodeType, (NodeId, Port)),
        end :(&'a NodeType, (NodeId, Port)),
        objs :Vec<(f32, ObjectId, &'a ObjectType)>,
    }

    #[derive(Debug)]
    struct DSwitch {
        trunk :Option<rolling_inf::NodeId>,
        left :Option<rolling_inf::NodeId>,
        right :Option<rolling_inf::NodeId>,
        side: Option<Side>,
    }

    for (track_id, Track { start_node, end_node, .. }) in inf.iter_tracks() {
        let (p1,n1) = if let Some(Node(p1, n1)) = inf.get_node(&start_node.0) { (p1,n1) } else { panic!() };
        let (p2,n2) = if let Some(Node(p2, n2)) = inf.get_node(&end_node.0) { (p2,n2) } else { panic!() };

        tracks.insert(track_id, OTrack {
            pos_start: *p1,
            pos_end: *p2,
            start: (n1, *start_node),
            end: (n2, *end_node),
            objs :Vec::new(),
        });
    }

    for (object_id,object) in inf.iter_objects() {
        let track = tracks.get_mut(&object.0).ok_or(format!("Invalid track ref"))?;
        track.objs.push((object.1, object_id, &object.2));
    }

    let mut dswitches = HashMap::new();
    for (node_id, node) in inf.iter_nodes() {
        if let Node(_,NodeType::Switch(_,_)) = node {
            dswitches.insert(node_id, DSwitch { trunk: None, left: None, right: None, side: None});
        }
    }

    for (track_id, mut t) in tracks {
        let (na, nb) = new_pair(&mut model.nodes);

        //
        // BEGIN NODE
        //
        match t.start {
            (NodeType::BufferStop, _) => {},
            (NodeType::Macro(name), (node_id,_)) => {
                model.nodes[na].edges = rolling_inf::Edges::ModelBoundary;
                entity_names.insert(RollingId::Node(na), EntityId::Node(node_id));
                //else { issues.push(DGraphConvertIssue::UnnamedBoundary(node_id)); }
            },
            (NodeType::Switch(Dir::Down, side), (sw_idx, Port { dir: Dir::Up, .. })) => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(na);
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (NodeType::Switch(Dir::Up, side), (sw_idx, Port { dir: Dir::Up, course: Some(c)})) => {
                match c {
                    Side::Left => {  dswitches.get_mut(&sw_idx).unwrap().left = Some(na); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(na); },
                };
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (NodeType::Switch(_,_),_) =>  panic!(),
            _ => unimplemented!(),
        }


        //
        // OBJECTS
        //

        t.objs.sort_by_key(|o| OrderedFloat(o.0));

        let mut last_node = nb;
        let mut last_pos = t.pos_start;

        for (pos,object_id,obj) in t.objs {
            let (na, nb) = new_pair(&mut model.nodes);
            match obj {
                ObjectType::Detector => {
                    detector_nodes.insert((na,nb));
                },
                ObjectType::Signal(dir) => {
                    let node_idx = match dir {
                        Dir::Up => nb,
                        Dir::Down => na,
                    };
                    let objid = new_object_id(&mut model.objects, rolling_inf::StaticObject::Signal);
                    model.nodes[node_idx].objects.push(objid);
                    entity_names.insert(RollingId::StaticObject(objid), EntityId::Object(object_id));
                },
                ObjectType::Balise(_) => {}, // not used in simulation, for now.
            }

            // set edge from last to na
            join_linear(&mut model.nodes, &mut edge_intervals, 
                        Interval { track: track_id, p1: last_pos, p2: pos },
                        last_node, na, (pos - last_pos) as _);

            last_pos = pos;
            last_node = nb;
        }


        //
        // END NODE
        //

        let (na, nb) = new_pair(&mut model.nodes);
        join_linear(&mut model.nodes, &mut edge_intervals,
                    Interval { track: track_id, p1: last_pos, p2: t.pos_end },
                    last_node, na, (t.pos_end - last_pos) as _);

        match t.end {
            (NodeType::BufferStop, _) => {},
            (NodeType::Macro(name), (node_id,_)) => {
                model.nodes[nb].edges = rolling_inf::Edges::ModelBoundary;
                entity_names.insert(RollingId::Node(nb), EntityId::Node(node_id));
                //else { issues.push(DGraphConvertIssue::UnnamedBoundary(node_id)); }
            },
            (NodeType::Switch(Dir::Up, side), (sw_idx, Port { dir :Dir::Down, ..})) => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(nb);
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (NodeType::Switch(Dir::Down, side), (sw_idx, Port { dir: Dir::Down, course :Some(c)})) => {
                match c {
                    Side::Left => {  dswitches.get_mut(&sw_idx).unwrap().left = Some(nb); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(nb); },
                };
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (NodeType::Switch(_,_),(idx,_)) =>  { issues.push(DGraphConvertIssue::MisconfiguredSwitch(idx)); },
            _ => unimplemented!(),
        }
    }


    //
    // RESOLVE SWITCHES
    //
    //

    for (i,s) in dswitches {
        println!("SWITCH {:?}", s);
        let trunk = s.trunk.ok_or(format!("Inconsistent switch data."))?;
        let left = s.left.ok_or(format!("Inconsistent switch data."))?;
        let right = s.right.ok_or(format!("Inconsistent switch data."))?;
        let side = s.side.ok_or(format!("Inconsistent switch data."))?;
        let side = match side {
            Side::Left =>  rolling_inf::SwitchPosition::Left,
            Side::Right => rolling_inf::SwitchPosition::Right,
        };

        let objid = new_object_id(&mut model.objects, rolling_inf::StaticObject::Switch {
                left_link: (left, 0.0),
                right_link: (right, 0.0),
                branch_side: side
        });

        let (na,nb) = new_pair(&mut model.nodes);
        // Join edges. They are all 0.0 m long so we don't need 
        // track interval tagging (for visualization)
        model.nodes[trunk].edges = rolling_inf::Edges::Single(na,    0.0);
        model.nodes[na   ].edges = rolling_inf::Edges::Single(trunk, 0.0);
        model.nodes[nb].edges = rolling_inf::Edges::Switchable(objid);
        model.nodes[left].edges = rolling_inf::Edges::Single(nb, 0.0);
        model.nodes[right].edges = rolling_inf::Edges::Single(nb, 0.0);
    }

    let tvd_sections = route_finder::detectors_to_sections(&mut model, &detector_nodes)?;

    // Call tvd section finder

    let dgraph = DGraph {
        rolling_inf: Arc::new(model),
        entity_names: entity_names,
        tvd_sections: tvd_sections,
        edge_intervals: edge_intervals,
        //boundary_names: boundary_names,
    };

    Ok((dgraph,issues))
}

