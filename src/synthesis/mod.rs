use numerical_optimization::powell::*;
use std::collections::{HashMap,HashSet, BTreeSet};
use boolinator::Boolinator;
use matches::*;
use nalgebra_glm as glm;
use log::*;

use crate::document;
use crate::document::model::*;
use crate::document::objects::*;
use crate::document::topology::*;
use crate::document::dgraph;
use crate::document::interlocking;

mod abstractdispatch;
mod initial;
mod optimize;
mod reduce;
mod add;
mod cost;

#[derive(Debug)]
pub enum FullSynMsg {
    S(String),
    TryingSignalSet(),
    ModelAvailable(String, f64, Design),
}

pub struct SynthesisBackground<'a> {
    pub topology :&'a Topology,
    pub plans :&'a [PlanSpec],
    pub vehicles :&'a [(usize,Vehicle)],
}

#[derive(Debug)]
pub enum SynErr { Aborted }

pub type Design = Vec<Object>;
pub type Object = (usize,f64,Function,Option<AB>);

pub use abstractdispatch::*;


pub fn full_synthesis( bg :&SynthesisBackground,
                       mut output :impl FnMut(FullSynMsg) -> bool) -> Result<(),SynErr> {
    output(FullSynMsg::S(format!("Starting full synthesis procedure."))).ok_or(SynErr::Aborted)?;
    let maximal_objects = initial::initial_design(&bg.topology);
    output(FullSynMsg::ModelAvailable(format!("Maximal model"), 0.0, 
                                      maximal_objects.clone())).ok_or(SynErr::Aborted)?;

    let mut signal_set_iterator = reduce::reduced_signal_sets(bg, maximal_objects);

    // Try all minimal signal sets
    // TODO reorg to breadth first?
    let mut n = 1;
    while let Some((mut design, adispatch)) = signal_set_iterator.next() {
        // the adispatch contains references to fixed infrastructure and
        // relative refernces to the Design, i.e. the objects whose positions can
        // be moved.
        info!("got plan set {:?}", adispatch);
        for d in adispatch.iter() {
            println!("A dispatch");
            for x in d.iter() {
                println!("  {:?}", x);
            }
        }
        output(FullSynMsg::TryingSignalSet()).ok_or(SynErr::Aborted)?;
        let (score,design) = optimize::optimize_locations(bg, &adispatch, &design);
        output(FullSynMsg::ModelAvailable(format!("reduced {}",n), score, design.clone())).ok_or(SynErr::Aborted)?;
        n += 1;

        let mut add_signal_iterator = add::add_signal(bg, design);
        while let Some(mut design) = add_signal_iterator.next() {
            output(FullSynMsg::TryingSignalSet()).ok_or(SynErr::Aborted)?;
            let (score,design) = optimize::optimize_locations(bg, &adispatch, &design);
            output(FullSynMsg::ModelAvailable(format!("added {}", n), score, design)).ok_or(SynErr::Aborted)?;
            n += 1;
        }
    }

    //output(FullSynMsg::S(format!("Done"))).ok_or(SynErr::Aborted)?;
    Ok(())
}


pub fn create_model(bg :&SynthesisBackground, design :&Vec<Object>) -> (Topology,dgraph::DGraph,interlocking::Interlocking) {
    let mut topo = (*bg.topology).clone();
    topo.trackobjects = topo.tracks.iter().map(|_| Vec::new()).collect::<Vec<_>>();
    for (obj_idx,(track_idx,pos,func,dir)) in design.iter().enumerate() {
        topo.trackobjects[*track_idx].push((*pos, glm::vec2(obj_idx as i32, 0), *func, *dir));
        if matches!(func, Function::MainSignal { .. }) {
            topo.trackobjects[*track_idx].push((*pos, glm::vec2(obj_idx as i32, 1), Function::Detector, None));
        }
    }

    let dgraph = dgraph::DGraphBuilder::convert(&topo).unwrap();
    let il = interlocking::calc(&dgraph);

    //println!("create_model interlocking");
    //for r in il.routes.iter() {
        //println!("route  {:?}", r);
    //}
    (topo,dgraph,il)
}
