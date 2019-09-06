use crate::synthesis::*;
use crate::document::history;
use crate::document::dgraph::DGraph;
use crate::document::interlocking::Interlocking;
use crate::synthesis::abstractdispatch::*;
use rolling::input::dispatch::DispatchAction;
use crate::document::history::convert_vehicle;
use crate::document::dispatch::max_time;
use crate::document::history::History;


pub fn measure(bg :&SynthesisBackground, allplans :&MultiPlan, design :&Design) -> (f64,f64) {
    let (topo,dgraph,il) = create_model(bg,design);
    let mut total_cost = 0.0;
    let mut total_travel = 0.0;

    for (planspec_id, dispatches) in allplans.iter().enumerate() {
        if dispatches.len() == 0 {
            total_cost += std::f64::INFINITY;
        } else {
            let mut planspec_cost = 0.0;
            for dispatch in dispatches.iter() {
                let commands = mk_commands(bg, &dgraph, &il, planspec_id, dispatch);
                let history = rolling::evaluate_plan(&dgraph.rolling_inf, 
                     &il.routes.iter().map(|r| r.route.clone()).enumerate().collect(),
                     &rolling::input::dispatch::Dispatch { actions: commands },
                     None);

                planspec_cost += max_time(&history);
                total_travel += traveled_length(&history);
            }
            total_cost += planspec_cost / dispatches.len() as f64;
        }
    }

    (total_cost,total_travel)
}

fn mk_commands(bg :&SynthesisBackground, dgraph :&DGraph, il:&Interlocking, 
               planspec_id :usize, abstract_dispatch :&AbstractDispatch) 
    -> Vec<DispatchAction<usize>> {

    let planspec = &bg.plans[planspec_id];
    abstract_dispatch.iter().flat_map(move |ad| {
        concrete_dispatch(dgraph, il, ad).into_iter().map(move |route_idx| {
            if il.routes[route_idx].route.entry.is_boundary() {
                let (vehicle_id,_) = planspec.trains.get(ad.train).unwrap();
                let (_,vehicle) = &bg.vehicles[vehicle_id.unwrap()];
                DispatchAction::Train(
                    format!("train{}", ad.train), 
                    convert_vehicle(&vehicle),
                    route_idx)
            } else {
                DispatchAction::Route(route_idx)
            }
        })
    }).collect()
}

fn traveled_length(h :&History) -> f64 {
    use rolling::output::history::*;
    use rolling::railway::dynamics::*;
    let mut l = 0.0;
    for (_,_,t) in h.trains.iter() {
        for e in t.iter() {
            match e {
                TrainLogEvent::Move(_,_,DistanceVelocity { dx, .. }) => { l += dx; },
                _ => {},
            }
        }
    }

    l
}

