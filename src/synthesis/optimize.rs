use nalgebra::DVector;
use crate::synthesis::*;

pub fn optimize_locations(bg :&SynthesisBackground, adispatch :&MultiPlan, design :&Design) -> (f64,Design) {
    use nalgebra::DVector;
    let (baseline_value, baseline_travel) = cost::measure(bg, adispatch, design);
    let start_pt = design_encode(bg, design);
    let (cost, best_pt) = powell_optimize_unit(design_encode(bg, design), |new_pt| {
        let (cost,travel) = cost::measure(bg, adispatch, &design_decode(bg, new_pt));
        cost
    }).unwrap();
    (cost, design_decode(bg, &best_pt))
}

fn design_encode(bg :&SynthesisBackground, design :&Design) -> DVector<f64> {

    unimplemented!()
}

fn design_decode(bg :&SynthesisBackground, pt: &DVector<f64>) -> Design {
    unimplemented!()
}
