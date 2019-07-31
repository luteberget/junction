#![allow(unused_imports)]
use rolling::input::staticinfrastructure as rolling_inf;
use std::collections::{HashMap, HashSet};
use ordered_float::OrderedFloat;
use std::sync::Arc;
use bimap::BiMap;
use crate::model::*;
use crate::objects::*;
use matches::matches;

pub type ModelNodeId = Pt;
pub type ModelObjectId = PtA;

#[derive(Debug)]
#[derive(Clone)]
pub struct DGraph {
    pub rolling_inf :Arc<rolling_inf::StaticInfrastructure>, 
    // separate Arc to be sent around to simulation threads

    pub node_ids : BiMap<ModelNodeId, rolling_inf::NodeId>,
    pub object_ids : BiMap<ModelObjectId, rolling_inf::ObjectId>,

    pub tvd_sections :HashMap<rolling_inf::ObjectId, 
        Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>>,

    pub edge_intervals :HashMap<(rolling_inf::NodeId, rolling_inf::NodeId), Interval>,
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Interval {
    // TODO ??? 
    //pub track :TrackId, 
    //pub p1 :Pos, // TODO ??? 
    //pub p2 :Pos,
}

pub struct DGraphBuilder {
    dgraph :rolling_inf::StaticInfrastructure,
}

impl DGraphBuilder {
    pub fn convert(model :&Model) -> Result<DGraph, ()> {
        let mut m = DGraphBuilder::new();

        // Create signals objects separately (they are not actually part of the "geographical" 
        // infrastructure network, they are merely pieces of state referenced by sight objects)
        let mut static_signals :HashMap<PtA, rolling_inf::ObjectId> = HashMap::new();
        for (p,o) in model.objects.iter().filter(|(p,o)| matches!(o.symbol.shape, Shape::Signal))  {
            let id = m.new_object(rolling_inf::StaticObject::Signal);
            static_signals.insert(*p,id);
        }

        // model needs to have a map from line segment to track index
        // List of tracks and related objects.
        let mut tracks = model./*railway.tracks*/linesegs.iter()
            .map(|(_segs,len)| (*len, Vec::new()))
            .collect::<Vec<(_,Vec<()>)>>();

//         // Calculate sight and put it onto track.
//         for sig in &static_signals {
//             let (track,pos,obj) = find_sight_point(model, sig);
//             let track = tracks.get_mut(&track));
//             track.1.push((*pos, None, obj));
//         }
// 
//         let mut nontrivial_nodes = HashMap::new();
//         for node in &nodes {
//             match node {
//                 Node::Switch => nontrivial.add
//                 // TODO Node::Crossing => nontrivial.add
//                 _ => {},
//             };
//         }
// 
//         let anonymous_detector = ObjectType::Detector;
// 
// 
//         for (track_id, mut t) in tracks {
//             // Go over each track and construct corresponding nodes for 
//             // internal objects.
// 
//             let (na,nb) = m.new_node_pair();
// 
//             if let NodeType::Macro(_) = t.start.0 {
//                 t.objs.push((0.0, None, &anonymous_detector));
//             }
// 
//             if let NodeType::Macro(_) = t.end.0 {
//                 t.objs.push((t.len, None, &anonymous_detector));
//             }
// 
//             match t.start {
//                 (NodeType::BufferStop, _ ) => {},
//             }
//         }
// 
// 
        unimplemented!()
    }

    pub fn new() -> DGraphBuilder {
        let model = rolling_inf::StaticInfrastructure {
            nodes: Vec::new(), 
            objects: Vec::new(),
        };
        DGraphBuilder { dgraph: model }
    }

    pub fn new_object(&mut self, obj :rolling_inf::StaticObject) -> rolling_inf::ObjectId {
        let id  = self.dgraph.objects.len();
        self.dgraph.objects.push(obj);
        id
    }

    pub fn new_nodes(&mut self) -> (rolling_inf::NodeId, rolling_inf::NodeId) {
        let (i1,i2) = (self.dgraph.nodes.len(), self.dgraph.nodes.len() +1);
        self.dgraph.nodes.push(rolling_inf::Node { other_node: i2,
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        self.dgraph.nodes.push(rolling_inf::Node { other_node: i1,
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        (i1,i2)
    }
}


