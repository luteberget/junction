use rolling::input::staticinfrastructure as rolling_inf;
use std::collections::{HashMap, HashSet};
use ordered_float::OrderedFloat;
use std::sync::Arc;
use bimap::BiMap;

pub type ModelNodeId = Pt;
pub type ModelObjectId = PtA;

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
    pub track :TrackId, 
    pub p1 :Pos, // TODO ??? 
    pub p2 :Pos,
}

pub struct DGraphBuilder {
    dgraph :rolling_inf::StaticInfrastructure,
}

impl DGraphBuilder {
    pub fn convert(model :&Model) -> Result<DGraph, ()> {
        let mut m = DGraphBuilder::new();
        let mut static_signals :HashMap<PtA, rolling_inf::ObjectId> = HashMap::new();
        for (p,o) in model.objects.iter().filter(|(p,o)|  {
        }
    }

    pub fn new() -> DGraphBuilder {
        let mut model = rolling_inf::StaticInfrastructure {
            nodes: Vec::new(), 
            objects: Vec::new(),
        };
        DGraphBuilder { dgraph: model }
    }
}


