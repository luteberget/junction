use matches::*;
use std::collections::HashMap;
use std::collections::BTreeSet;
use rolling::input::staticinfrastructure as rolling_inf;
use nalgebra_glm as glm;

use crate::synthesis::*;
use crate::synthesis::abstractdispatch::*;
use crate::document::topology;
use crate::document::dgraph;
use crate::document::interlocking;
use crate::document::plan;

pub fn reduced_signal_sets<'a>(bg :&'a SynthesisBackground, design :Design) 
    -> impl Iterator<Item = (Design, MultiPlan)> + 'a {
        
    let (topo,dgraph,il) = create_model(bg, &design);
    let inf = plan::convert_inf(&il.routes.iter()
                                .map(|i| i.route.clone()).enumerate().collect());
    let plans = bg.plans.iter().map(|p| plan::convert_plan(&il, bg.vehicles, p))
        .collect::<Result<Vec<_>,_>>().unwrap();

    println!("create optmizer");
    let mut optimizer = planner::optimize::SignalOptimizer::new(inf, plans.into());
    println!("create optmizer ok");

    Iter { bg, topo, dgraph, il, optimizer }
}

pub struct Iter<'a> {
    bg :&'a SynthesisBackground<'a>,
    topo :topology::Topology,
    dgraph :dgraph::DGraph,
    il :interlocking::Interlocking,
    optimizer :planner::optimize::SignalOptimizer,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Design,MultiPlan);
    fn next(&mut self) -> Option<(Design,MultiPlan)> {
        let opt = &mut self.optimizer;
        let topo = &self.topo;
        let dgraph = &self.dgraph;
        let il = &self.il;
        opt.next_signal_set().map(|mut s| {
            let (design, id_map) = convert_signals(topo, dgraph, s.get_signals());
            let dispatches = s.get_dispatches().into_iter().enumerate()
                .map(|(planspec_idx,routeplans)| routeplans.into_iter()
                     .map(|routeplan| abstract_dispatches(dgraph, il, &id_map, &routeplan))
                     .collect()).collect();
            (design, dispatches)
        })
    }
}


fn convert_signals(topo :&Topology, dgraph :&dgraph::DGraph, 
                   signals :&HashMap<planner::input::SignalId, bool>) 
    -> (Design,HashMap<PtA,PtA>) {

    let mut design = Vec::new();
    // maximal_design_object_id --> minimal_design_object_id 
    let mut id_map = HashMap::new();

    let pt_id = dgraph.object_ids.iter().map(|(a,b)| (*b,*a))
        .collect::<HashMap<PtA, rolling_inf::ObjectId>>();

    for (track_idx, track_objects) in topo.trackobjects.iter().enumerate() {
        for (pos, id, func, dir) in track_objects.iter() {
            let active = pt_id.get(id).map(|o| planner::input::SignalId::ExternalId(*o))
                                .and_then( |o| signals.get(&o)).unwrap_or(&false);
            if *active || matches!(func, Function::Detector) {
                id_map.insert(glm::vec2(id.x as _, 0) , glm::vec2(design.len() as _, 0));
                design.push((track_idx,*pos,*func,*dir));
            }
        }
    }
    (design,id_map)
}
