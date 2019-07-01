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
}


