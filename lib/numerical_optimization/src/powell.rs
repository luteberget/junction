use nalgebra::DVector;
use log::*;
use crate::brent::*;

#[derive(Debug)]
pub enum PowellErr {
    InvalidInitialPoint,
}

fn is_unit_box(pt :&DVector<f64>) -> Result<(), PowellErr> {
    for x in pt.iter() {
        if !(0.0 <= *x) || !(*x <= 1.0) {
            return Err(PowellErr::InvalidInitialPoint);
        }
    }
    Ok(())
}

fn unit_box_parameter_bounds(point :&DVector<f64>, vector :&DVector<f64>) -> (f64,f64) {
    let mut max_alpha = std::f64::INFINITY;
    let mut min_alpha = -std::f64::INFINITY;

    for (x0,v0) in point.iter().cloned().zip(vector.iter().cloned()) {
        if v0 > 0.0 {
            max_alpha = max_alpha.min((1.0-x0)/v0);
        } else if v0 < 0.0 {
            min_alpha = min_alpha.max((1.0-x0)/v0);
        }

        if v0 > 0.0 {
            min_alpha = min_alpha.max(-x0/v0);
        } else if v0 < 0.0 {
            max_alpha = max_alpha.min(-x0/v0);
        }
    }

    (min_alpha, max_alpha)
}

pub fn powell_optimize_unit(initial_point :DVector<f64>,
                            mut point_cost :impl FnMut(&DVector<f64>) -> f64) 
    -> Result<(f64, DVector<f64>),PowellErr> {
    trace!("Powell optimize unit start");

    let cost_improvement_threshold = 0.1;

    // Check that the initial vector is inside the unit cube.
    is_unit_box(&initial_point)?;

    let initial_cost = point_cost(&initial_point);
    let mut powell_point = initial_point;
    let mut powell_cost = initial_cost;

    let mut search_vectors = (0..(powell_point.len())).map(|i| {
        let mut v = DVector::from_element(powell_point.len(), 0.0);
        v[i] = 0.0; v}).collect::<Vec<_>>();

    loop {
        trace!("Powell iteration");
        let mut iter_point = powell_point.clone();
        let mut iter_start_point = powell_point.clone();
        let mut iter_cost = powell_cost;
        let mut best_search_vector :Option<(usize, f64)> = None;
        for (v_i,v) in search_vectors.iter().enumerate() {
            let (min_alpha, max_alpha) = unit_box_parameter_bounds(&iter_point, v);
            let (alpha, brent_cost) = brent_minimum( 
                |alpha| {
                    point_cost(&(iter_point.clone() + alpha * v))
                },
                min_alpha, 0.0, max_alpha, 32, None);

            let brent_improvement = iter_cost - brent_cost;
            if brent_improvement > cost_improvement_threshold {
                iter_cost = brent_cost;
                iter_point += alpha*v;
                is_unit_box(&iter_point)?;
                if best_search_vector.is_none() || best_search_vector.unwrap().1 > brent_cost {
                    best_search_vector = Some((v_i, brent_cost));
                }
            } else {
                debug!("No brent improvement.");
            }
        }

        let iter_offset = iter_point.clone() - iter_start_point;
        let iter_improvement = powell_cost - iter_cost;

        if iter_improvement > cost_improvement_threshold {
            debug!("Powell: iter imrovement.");
            search_vectors.remove(best_search_vector.unwrap().0);
            search_vectors.push(iter_offset.normalize());
            powell_point = iter_point;
            powell_cost = iter_cost;
        } else {
            debug!("Powell: no iter improvement, finished.");
            break;
        }
    }

    Ok((powell_cost, powell_point))
}


