use log::*;
use ordered_float::OrderedFloat;
use nalgebra::DVector;
use crate::synthesis::*;
use itertools::*;
use std::mem::replace;
use permutation::Permutation;

pub fn optimize_locations(bg :&SynthesisBackground, adispatch :&MultiPlan, design :&Design) -> (f64,Design) {
    info!("optimize_locations: starting");
    let order = permutation::sort_by_key(&design[..], 
                 |(tr,pos,_,_)| (*tr, OrderedFloat(*pos)));
    let baseline_value = cost::measure(bg, adispatch, design);
    let mut n = 0;
    let start_pt = design_encode(bg, design, &order);
    println!("Encoding first design {:?}\n  {:?}", design, start_pt);
    let (cost, best_pt) = powell_optimize_unit(start_pt, |new_pt| {
        n += 1;
        cost::measure(bg, adispatch, &design_decode(bg, new_pt, design, &order))
    }).unwrap();
    println!("optimize_locations: {} iterations", n);
    (cost, design_decode(bg, &best_pt, design, &order))
}

fn design_encode(bg :&SynthesisBackground, design :&Design, order :&Permutation) 
    -> DVector<f64> {

    let min_dist = 21.9;

    DVector::from_iterator(design.len(), 
       (0..(design.len())).map(|i| &design[order.apply_inv_idx(i)])
        .group_by(|(tr,_,_,_)| tr).into_iter().flat_map(|(tr,group)| {
           let track_length = bg.topology.tracks[*tr].0;
           group.scan(0.0, move |prev, (_,pos,_,_)| {
               Some(linearstep(replace(prev, *pos) + min_dist, track_length - min_dist, *pos))
           })
       })
    )
}

fn design_decode(bg :&SynthesisBackground, pt: &DVector<f64>, design :&Design, order :&Permutation) -> Design {

    let min_dist = 21.9;

    let out = pt.iter().enumerate().map(|(i,ipos)| (&design[order.apply_inv_idx(i)], ipos))
        .group_by(|((tr,_,_,_),_)| tr).into_iter().flat_map(|(tr,group)| {
            let track_length = bg.topology.tracks[*tr].0;
            // we have [0,1], and we want to map it to [last_pos,track_length]
            group.scan(0.0, move |prev, ((_tr,_oldpos,func,dir),ipos)| {
                let pos = glm::lerp_scalar(*prev + min_dist, track_length - min_dist, *ipos);
                *prev = pos; Some((*tr,pos,*func,*dir))
            })
        }).collect::<Vec<_>>();
    out
}

fn linearstep(lo :f64, hi :f64, val :f64) -> f64 {
    (val - lo)/(hi - lo)
}
