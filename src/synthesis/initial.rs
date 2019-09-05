use crate::document::topology::*;
use crate::document::model::*;
use crate::document::objects::*;
use crate::synthesis::*;

/// An initial guess for signal and detector placement.
/// Places a signal at 0 and 150 m from every switch.
pub fn initial_design(topo :&Topology) -> Design {

    let stock_length = 23.0;
    let fouling_length = 50.0;
    let overlap_lengths = vec![0.0, 150.0];

    let mut objects = Vec::new();

    for (track_idx,(length,(_,port_a),(_,port_b))) in topo.tracks.iter().enumerate()  {
        for (pos_fn, port, dir) in &[(&(|x| x)          as &Fn(f64) -> f64, port_a, AB::B),
                                     (&(|x| length - x) as &Fn(f64) -> f64, port_b, AB::A)] {
            match port {
                Port::Trunk => { // set a detector at the stock
                    objects.push((track_idx, pos_fn(stock_length), Function::Detector, None)); }, //
                Port::Left | Port::Right => { // set a signal and detector at each overlap length
                    objects.push((track_idx, pos_fn(fouling_length), Function::Detector, None)); 
                    let sig = Function::MainSignal { has_distant: true };
                    objects.push((track_idx, pos_fn(fouling_length), sig, Some(*dir))); 
                },

                _ => {}, // TODO crossings
            }
        }
    }

    objects
}
