use crate::synthesis::*;
use crate::document::history;
use crate::document::dgraph::DGraph;
use crate::document::interlocking::Interlocking;
use crate::synthesis::abstractdispatch::*;
use rolling::input::dispatch::DispatchAction;
use crate::document::history::convert_vehicle;
use crate::document::dispatch::max_time;
use crate::document::plan::eval_plan;
use crate::document::history::History;


pub fn measure_dispatch(bg :&SynthesisBackground, dgraph :&DGraph, il :&Interlocking,
    planspec_id :usize, dispatch :&AbstractDispatch) -> Result<f64,()> {

    let commands = mk_commands(bg, &dgraph, &il, planspec_id, dispatch)?;
    //println!("Testing c {:?}", commands);
    let history = rolling::evaluate_plan(&dgraph.rolling_inf, 
         &il.routes.iter().map(|r| r.route.clone()).enumerate().collect(),
         &rolling::input::dispatch::Dispatch { actions: commands },
         None);
    if eval_plan(&dgraph, &bg.plans[planspec_id], &history).is_ok() {
        Ok(max_time(&history))
    } else {
         Err(())
    }
}

pub fn measure(bg :&SynthesisBackground, allplans :&MultiPlan, design :&Design) -> f64 {
    //println!("cost::measure");
    let (topo,dgraph,il) = create_model(bg,design);
    let mut total_cost = 0.0;
    //println!("Testing design {:?}", design);
    //println!("Testing design on plans {:?}", allplans);

    for (planspec_id, dispatches) in allplans.iter().enumerate() {
        if dispatches.len() == 0 {
            total_cost += std::f64::INFINITY;
        } else {
            let planspec_cost = dispatches.iter().map(|d|  {
                //println!("measure on dispatch {:?}", d);
                    measure_dispatch(bg, &dgraph, &il, planspec_id, d)
                    .unwrap_or(std::f64::INFINITY)}).sum::<f64>();
            total_cost += planspec_cost / dispatches.len() as f64;
        }
    }

    total_cost
}

fn mk_commands(bg :&SynthesisBackground, dgraph :&DGraph, il:&Interlocking, 
               planspec_id :usize, abstract_dispatch :&AbstractDispatch) 
    -> Result<Vec<DispatchAction<usize>>,()> {

    let planspec = &bg.plans[planspec_id];
    let mut output = Vec::new();
    for acmd in abstract_dispatch.iter() {
        for route_idx in concrete_dispatch(dgraph, il, acmd)? {
            if il.routes[route_idx].route.entry.is_boundary() {
                let (vehicle_id,_) = planspec.trains.get(acmd.train).unwrap();
                let (_,vehicle) = &bg.vehicles[vehicle_id.unwrap()];
                output.push(DispatchAction::Train(
                    format!("train{}", acmd.train), 
                    convert_vehicle(&vehicle),
                    route_idx));
            } else {
                output.push(DispatchAction::Route(route_idx));
            }
        }
    }
    Ok(output)
}

