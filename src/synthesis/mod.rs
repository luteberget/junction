use numerical_optimization::powell::*;
use std::collections::HashMap;
use boolinator::Boolinator;

use crate::document;
use crate::document::model::*;
use crate::document::objects::*;
use crate::document::topology::*;

mod initial;
mod optimize;
mod reduce;
mod add;

pub enum FullSynMsg {
    S(String),
    TryingSignalSet(),
    ModelAvailable(String, f64, Design),
}

pub struct SynthesisBackground<'a> {
    topo :&'a Topology,
    plans :&'a [PlanSpec],
    vehicles :&'a [Vehicle],
}

pub enum SynErr { Aborted }
pub type Design = Vec<Object>;
pub type Object = (usize,f64,Function,Option<AB>);

pub fn full_synthesis( bg :&SynthesisBackground,
                       mut output :impl FnMut(FullSynMsg) -> bool) -> Result<(),SynErr> {
    output(FullSynMsg::S(format!("Starting full synthesis procedure."))).ok_or(SynErr::Aborted)?;
    let maximal_objects = initial::initial_design(&bg.topo);
    output(FullSynMsg::ModelAvailable(format!("Maximal model"), 0.0, 
                                      maximal_objects.clone())).ok_or(SynErr::Aborted)?;

    let mut signal_set_iterator = reduce::reduced_signal_sets(bg, maximal_objects);
    // Try all minimal signal sets
    // TODO reorg to breadth first?
    let mut n = 1;
    while let Some(mut design) = signal_set_iterator.next() {
        output(FullSynMsg::TryingSignalSet()).ok_or(SynErr::Aborted)?;
        let score = optimize::optimize_locations(bg, &mut design);
        output(FullSynMsg::ModelAvailable(format!("reduced {}",n), score, design.clone())).ok_or(SynErr::Aborted)?;
        n += 1;

        let mut add_signal_iterator = add::add_signal(bg, design);
        while let Some(mut design) = add_signal_iterator.next() {
            output(FullSynMsg::TryingSignalSet()).ok_or(SynErr::Aborted)?;
            let score = optimize::optimize_locations(bg, &mut design);
            output(FullSynMsg::ModelAvailable(format!("added {}", n), score, design)).ok_or(SynErr::Aborted)?;
            n += 1;
        }
    }

    Ok(())
}

