use matches::matches;
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

    //println!("create optmizer");
    let mut optimizer = planner::optimize::SignalOptimizer::new(inf, plans.into());
    //println!("create optmizer ok");

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
            let dispatches = s.get_dispatches();
            let detectors = s.reduce_detectors(&dispatches);
            let signals = s.get_signals();
            println!("SIGNALS");
            println!("{:?}", signals);
            let (design, id_map) = convert_signals(topo, dgraph, &signals, &detectors);
            let dispatches = dispatches.into_iter().enumerate()
                .map(|(planspec_idx,routeplans)| routeplans.into_iter()
                     .map(|routeplan| abstract_dispatches(dgraph, il, &id_map, &routeplan))
                     .collect()).collect();
            (design, dispatches)
        })
    }
}


fn convert_signals(topo :&Topology, dgraph :&dgraph::DGraph, 
                   signals :&HashSet<planner::input::SignalId>, 
                   detectors :&HashSet<planner::input::SignalId>) 
    -> (Design,HashMap<PtA,PtA>) {

    let mut design = Vec::new();
    // maximal_design_object_id --> minimal_design_object_id 
    let mut id_map = HashMap::new();

    let sig_id = dgraph.object_ids.iter().map(|(a,b)| (*b,*a))
        .collect::<HashMap<PtA, rolling_inf::ObjectId>>();
    let canonical_node_id = |n| n/2*2;
    let det_id = dgraph.detector_ids.iter().map(|(a,b)| (*b,canonical_node_id(*a)))
        .collect::<HashMap<PtA, rolling_inf::NodeId>>();

    //println!("all signals {:?}", sig_id);
    //println!("all detectors {:?}", det_id);

    //println!("spec signals {:?}", signals);
    //println!("spec detectors {:?}", detectors);

    for (track_idx, track_objects) in topo.trackobjects.iter().enumerate() {
        for (pos, id, func, dir) in track_objects.iter() {
            //println!("convert {:?}", (pos,id,func,dir));
            match func {
                Function::Detector => {
                    if det_id.get(id).map(|d| detectors.contains(&planner::input::SignalId::Detector(*d)) ||
                                              detectors.contains(&planner::input::SignalId::Detector(*d + 1)))
                        .unwrap_or(false) {

                        design.push((track_idx, *pos, *func, *dir));
                        id_map.insert(glm::vec2(id.x as _, 0) , glm::vec2(design.len() as _, 0));
                    }
                },
                Function::MainSignal { .. } => {
                    if sig_id.get(id).map(|o| signals.contains(&planner::input::SignalId::Signal(*o)))
                        .unwrap_or(false) {

                            
                        id_map.insert(glm::vec2(id.x as _, 0) , glm::vec2(design.len() as _, 0));
                        design.push((track_idx, *pos, *func, *dir));

                    } else if sig_id.get(id).map(|o| detectors.contains(&planner::input::SignalId::Signal(*o)))
                        .unwrap_or(false) {

                        id_map.insert(glm::vec2(id.x as _, 0) , glm::vec2(design.len() as _, 0));
                        design.push((track_idx, *pos, Function::Detector, None));
                    }
                },
            }
        }
    }
    (design,id_map)
}
