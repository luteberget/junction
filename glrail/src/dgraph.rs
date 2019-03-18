use rolling::input::staticinfrastructure::{
    Dist, NodeId, ObjectId,
    StaticInfrastructure, Node as DNode, Edges, StaticObject, SwitchPosition };
use crate::infrastructure::{ EntityId, Port, Side, Dir, Track, Node, Object, Entity };
use crate::interlocking::{ Route };
use std::collections::HashMap; 
use std::collections::HashSet; 
use ordered_float::OrderedFloat;

// TODO move to route derive crate
pub use railml2dgraph::routes::ConvertRouteIssue;

#[derive(Debug)]
pub struct DGraph {
    /// static infrastructure with internal indexing (names do not correspond to glrail entities)
    pub rolling_inf : StaticInfrastructure,
    /// Reference from rolling dgraph indices to glrail entitiy vec
    pub entity_names : HashMap<EntityId, usize>,
    /// tvd sections mapped to set of edges
    pub tvd_sections :HashMap<ObjectId, Vec<(NodeId,NodeId)>>,
    /// edges mapped to track interval (track in glrail model)
    pub edge_intervals : HashMap<(NodeId,NodeId),Interval>,
    /// string names of boundaries, to be referred to in dispatch plans
    pub boundary_names :HashMap<String, NodeId>,
}

impl Default for DGraph {
    fn default() -> Self {
        DGraph {
            rolling_inf: StaticInfrastructure { nodes: vec![], objects: vec![] },
            entity_names: HashMap::new(),
            tvd_sections: HashMap::new(),
            edge_intervals: HashMap::new(),
            boundary_names: HashMap::new(),
        }
    }
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Interval {
    pub track_idx :EntityId,
    pub p1 :f32,
    pub p2 :f32,
}


pub fn make_routes(dgraph :&DGraph) -> (Vec<Route>, Vec<ConvertRouteIssue>) {
    // TODO
    (vec![],vec![])
}

pub enum DGraphConvertIssue {
    UnnamedBoundary(EntityId),
    MisconfiguredSwitch(EntityId),
}


pub fn convert_entities(ent :&Vec<Option<Entity>>) -> Result<(DGraph,Vec<DGraphConvertIssue>), String>  {
    let mut issues = Vec::new();
    let mut tracks = HashMap::new();
    let mut entity_names = HashMap::new();
    let mut edge_intervals = HashMap::new();
    let mut detector_nodes = HashSet::new();
    let mut boundary_names = HashMap::new();

    let mut model = StaticInfrastructure {
        nodes: Vec::new(),
        objects: Vec::new(),
    };

    fn new_pair(nodes :&mut Vec<DNode>) -> (NodeId,NodeId) {
        let (i1,i2) = (nodes.len(), nodes.len()+1);
        nodes.push(DNode { other_node: i2, edges: Edges::Nothing, objects: Default::default() });
        nodes.push(DNode { other_node: i1, edges: Edges::Nothing, objects: Default::default() });
        (i1,i2)
    }

    fn new_object_id(objects :&mut Vec<StaticObject>, obj :StaticObject) -> ObjectId {
        let idx = objects.len();
        objects.push(obj);
        idx
    }

    fn join_linear(nodes :&mut Vec<DNode>, 
                   edge_intervals :&mut HashMap<(NodeId,NodeId),Interval>, 
                   interval: Interval, i1 :NodeId, i2 :NodeId, dist :f64) {
        nodes[i1].edges = Edges::Single(i2, dist);
        nodes[i2].edges = Edges::Single(i1, dist);
        edge_intervals.insert((i1,i2),interval);
        edge_intervals.insert((i2,i1),interval);
    }

    struct OTrack<'a> {
        pos_start :f32,
        pos_end :f32,
        start :(&'a Node, (usize, Port)),
        end :(&'a Node, (usize, Port)),
        objs :Vec<(f32, EntityId, &'a Object)>,
    }

    #[derive(Debug)]
    struct DSwitch {
        trunk :Option<NodeId>,
        left :Option<NodeId>,
        right :Option<NodeId>,
        side: Option<Side>,
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
            track.objs.push((*p,i,obj));
        }
    }

    let mut dswitches = HashMap::new();
    for (i,e) in ent.iter().enumerate() {
        if let Some(Entity::Node(p,Node::Switch(_,_))) = e {
            dswitches.insert(i, DSwitch { trunk: None, left: None, right: None, side: None});
        }
    }


    for (track_idx, mut t) in tracks {
        let (na, nb) = new_pair(&mut model.nodes);

        //
        // BEGIN NODE
        //
        match t.start {
            (Node::BufferStop, _) => {},
            (Node::Macro(name), _) => {
                model.nodes[na].edges = Edges::ModelBoundary;
                if let Some(name) = name { boundary_names.insert(name.clone(),na); }
            },
            (Node::Switch(Dir::Down, side), (sw_idx, Port { dir: Dir::Up, .. })) => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(na);
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (Node::Switch(Dir::Up, side), (sw_idx, Port { dir: Dir::Up, course: Some(c)})) => {
                match c {
                    Side::Left => {  dswitches.get_mut(&sw_idx).unwrap().left = Some(na); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(na); },
                };
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (Node::Switch(_,_),_) =>  panic!(),
            _ => unimplemented!(),
        }


        //
        // OBJECTS
        //

        t.objs.sort_by_key(|o| OrderedFloat(o.0));

        let mut last_node = nb;
        let mut last_pos = t.pos_start;

        for (pos,eid,obj) in t.objs {
            let (na, nb) = new_pair(&mut model.nodes);
            match obj {
                Object::Detector => {
                    detector_nodes.insert((na,nb));
                },
                Object::Signal(dir) => {
                    let node_idx = match dir {
                        Dir::Up => nb,
                        Dir::Down => na,
                    };
                    let objid = new_object_id(&mut model.objects, StaticObject::Signal);
                    model.nodes[node_idx].objects.push(objid);
                    entity_names.insert(eid,objid);
                },
                Object::Balise(_) => {}, // not used in simulation, for now.
            }

            // set edge from last to na
            join_linear(&mut model.nodes, &mut edge_intervals, 
                        Interval { track_idx, p1: last_pos, p2: pos },
                        last_node, na, (pos - last_pos) as _);

            last_pos = pos;
            last_node = nb;
        }


        //
        // END NODE
        //

        let (na, nb) = new_pair(&mut model.nodes);
        join_linear(&mut model.nodes, &mut edge_intervals,
                    Interval { track_idx, p1: last_pos, p2: t.pos_end },
                    last_node, na, (t.pos_end - last_pos) as _);

        match t.end {
            (Node::BufferStop, _) => {},
            (Node::Macro(name), (idx,_)) => {
                model.nodes[nb].edges = Edges::ModelBoundary;
                if let Some(name) = name { boundary_names.insert(name.clone(),na); }
                else { issues.push(DGraphConvertIssue::UnnamedBoundary(idx)); }
            },
            (Node::Switch(Dir::Up, side), (sw_idx, Port { dir :Dir::Down, ..})) => {
                dswitches.get_mut(&sw_idx).unwrap().trunk = Some(nb);
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (Node::Switch(Dir::Down, side), (sw_idx, Port { dir: Dir::Down, course :Some(c)})) => {
                match c {
                    Side::Left => {  dswitches.get_mut(&sw_idx).unwrap().left = Some(nb); },
                    Side::Right => { dswitches.get_mut(&sw_idx).unwrap().right = Some(nb); },
                };
                dswitches.get_mut(&sw_idx).unwrap().side = Some(*side);
            },
            (Node::Switch(_,_),(idx,_)) =>  { issues.push(DGraphConvertIssue::MisconfiguredSwitch(idx)); },
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
            Side::Left => SwitchPosition::Left,
            Side::Right => SwitchPosition::Right,
        };

        let objid = new_object_id(&mut model.objects, StaticObject::Switch {
                left_link: (left, 0.0),
                right_link: (right, 0.0),
                branch_side: side
        });

        let (na,nb) = new_pair(&mut model.nodes);
        // Join edges. They are all 0.0 m long so we don't need 
        // track interval tagging (for visualization)
        model.nodes[trunk].edges = Edges::Single(na,    0.0);
        model.nodes[na   ].edges = Edges::Single(trunk, 0.0);
        model.nodes[nb].edges = Edges::Switchable(objid);
        model.nodes[left].edges = Edges::Single(nb, 0.0);
        model.nodes[right].edges = Edges::Single(nb, 0.0);
    }

    let tvd_sections = route_finder::detectors_to_sections(&mut model, &detector_nodes)?;

    // Call tvd section finder

    let dgraph = DGraph {
        rolling_inf: model,
        entity_names: entity_names,
        tvd_sections: tvd_sections,
        edge_intervals: edge_intervals,
        boundary_names: boundary_names,
    };

    Ok((dgraph,issues))
}

