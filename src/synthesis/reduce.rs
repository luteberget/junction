use std::collections::HashMap;
use std::collections::BTreeSet;
use rolling::input::staticinfrastructure as rolling_inf;
use nalgebra_glm as glm;

use crate::synthesis::*;
use crate::document::topology;
use crate::document::dgraph;
use crate::document::interlocking;
use crate::document::plan;

pub fn reduced_signal_sets(bg :&SynthesisBackground, design :Design) 
    -> impl Iterator<Item = (Design, MultiPlan)> {
        
    let (topo,dgraph,il) = create_model(bg, &design);
    let inf = plan::convert_inf(&il.routes.iter().map(|i| i.route.clone()).enumerate().collect());
    let plans = bg.plans.iter().map(|p| plan::convert_plan(&il, bg.vehicles, p)).collect::<Result<Vec<_>,_>>()
        .unwrap();

    let mut optimizer = planner::optimize::SignalOptimizer::new(inf, plans.into());

    Iter { topo, dgraph, optimizer }
}

pub struct Iter {
    topo :topology::Topology,
    dgraph :dgraph::DGraph,
    optimizer :planner::optimize::SignalOptimizer,
}

impl Iterator for Iter {
    type Item = (Design,MultiPlan);
    fn next(&mut self) -> Option<(Design,MultiPlan)> {
        let opt = &mut self.optimizer;
        let topo = &self.topo;
        let dgraph = &self.dgraph;
        opt.next_signal_set().map(|s| {
            let design = convert_signals(topo, dgraph, s.get_signals());
            (design, unimplemented!())
        })
    }
}


fn convert_signals(topo :&Topology, dgraph :&dgraph::DGraph, signals :&HashMap<planner::input::SignalId, bool>) -> Design {
    let mut design = Vec::new();
    let pt_id = dgraph.object_ids.iter().map(|(a,b)| (*b,*a)).collect::<HashMap<PtA, rolling_inf::ObjectId>>();

    for (track_idx, track_objects) in topo.trackobjects.iter().enumerate() {
        for (pos, id, func, dir) in track_objects.iter() {
            let active = pt_id.get(id).map(|o| planner::input::SignalId::ExternalId(*o))
                                .and_then( |o| signals.get(&o)).unwrap_or(&false);
            if *active {
                design.push((track_idx,*pos,*func,*dir));
            }
        }
    }

    design
}


