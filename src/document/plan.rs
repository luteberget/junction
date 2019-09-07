use std::collections::{HashMap, HashSet};
use crate::document::interlocking::*;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::document::model::*;
use crate::document::history;
use crate::document::dgraph::DGraph;
use rolling::output::history::*;

#[derive(Debug)]
pub enum ConvertPlanErr {
    VehicleRefMissing,
    VehicleMissing,
}

pub enum TestPlanErr {
    MissingVisits
}

pub fn eval_plan(dgraph :&DGraph, plan_spec :&PlanSpec, history :&History) -> Result<(), TestPlanErr> {


    //
    // 1. check train's visits
    for (train_idx, (train_id, (veh, visits))) in plan_spec.trains.iter().enumerate() {
        let mut current_visit = 0;
        let (train_name, train_params, train_log) = &history.trains[train_idx];
        for ev in train_log.iter() {
            if event_matches_spec(dgraph, &visits.data()[current_visit].1, ev) {
                current_visit += 1;
                if !(current_visit < visits.data().len()) { 
                    break;
                }
            }
        }
        if current_visit < visits.data().len() {
            // train train_idx failed to reach its planned locations
            return Err(TestPlanErr::MissingVisits); 
        }
    }

    // 2. check ordering constraints and time diff
    for (ra,rb,dt) in plan_spec.order.iter() {
        // TODO
    }

    Ok(())
}

pub fn test_plan(dgraph :&DGraph,
                 il :&Interlocking, 
                 vehicles :&[(usize,Vehicle)],
                 plan_spec :&PlanSpec,
                 candidate :&planner::input::RoutePlan) 
    -> Result<Result<(Dispatch, History),TestPlanErr>,String> {
    let dispatch = convert_dispatch_commands(candidate, il, plan_spec)?;

    // simulate the dispatch
    let (history,route_refs) =
         history::get_history(vehicles, &dgraph.rolling_inf, il, &dispatch.commands)?;

    // then check that the plan is satisfied
    if let Err(e) = eval_plan(dgraph, plan_spec, &history) { return Ok(Err(e)); }
    return Ok(Ok((dispatch,history)));
}

fn event_matches_spec(dgraph :&DGraph, visit :&Visit, event :&TrainLogEvent) -> bool {
    if let TrainLogEvent::Node(n) = event {
        for l in visit.locs.iter() {
            match l {
                Ok(Ref::Node(pt)) => {
                    // have to check both boundaries and switch nodes in the dgraph
                    if dgraph.node_ids.get_by_left(n) == Some(pt) { return true; }
                    if dgraph.switch_ids.get_by_left(n) == Some(pt) { return true; }
                },
                _ => unimplemented!(), // TODO
            };
        }
    }
    false
}

pub fn get_dispatches(
      dgraph :&DGraph,
      il :&Interlocking, 
      vehicles :&[(usize,Vehicle)],
      plan :&PlanSpec,
      ) -> Result<Vec<(Dispatch, History)>, String> {

    let routes : HashMap<usize,rolling_inf::Route> = 
        il.routes.iter().map(|r| r.route.clone()).enumerate().collect();
    let route_specs : HashMap<usize,RouteSpec> = 
        il.routes.iter().map(|r| r.id.clone()).enumerate().collect();

    let plan_inf = convert_inf(&routes);
    let plan_usage = convert_plan(il, vehicles, plan).
        map_err(|e| format!("{:?}", e))?;
    let config = planner::input::Config {
        n_before: 3, n_after: 3, exact_n: None, optimize_signals: false,
    };

    //println!(" STARTIN GPLANNIGN");
    //println!("infrastructure {:#?}", plan_inf);
    //println!("usage {:#?}", plan_usage);

    let mut output = Vec::new();
    planner::solver::plan(&config, &plan_inf, &plan_usage, |candidate| {
        println!("got one plan");
        if let Ok((d,p)) = test_plan(dgraph, il, vehicles, plan, candidate).unwrap() {
            output.push((d,p));
        }
        false
    });
    println!("planner finished");
    Ok(output)
}


fn convert_dispatch_commands(routeplan :&planner::input::RoutePlan, il :&Interlocking,
                          plan :&PlanSpec) -> Result<Dispatch,String> {

    use std::collections::BTreeSet;

    let mut commands = Vec::new();
    let mut last_active_routes = BTreeSet::new();

    for state in routeplan.iter() {
        let active_routes = state.iter().filter_map(|((elementary,part),train_id)| {
            // use partial as representative for elementary route
            if *part == 0 && train_id.is_some() {
                Some((*elementary, train_id.unwrap())) 
            } else { None }
        }).collect::<BTreeSet<_>>();

        for (new_route, train_id) in active_routes.difference(&last_active_routes) {
            // check if the route is in the birth of a train (comes from boundary)
            match il.routes[*new_route].route.entry {
                rolling_inf::RouteEntryExit::Boundary(_) => {
                    // Spawn new train
                    commands.push((0.0, Command::Train(
                                plan.trains.get(*train_id).unwrap().0.unwrap(), //vehicle id
                                il.routes[*new_route].id)));
                },
                rolling_inf::RouteEntryExit::Signal(_) 
                    | rolling_inf::RouteEntryExit::SignalTrigger { .. } => {
                        commands.push((0.0, Command::Route(il.routes[*new_route].id)));
                },
            }
        }

        last_active_routes = active_routes;
    }

    Ok(Dispatch::from_vec(commands))
}


pub fn convert_inf(routes :&rolling_inf::Routes<usize>) -> planner::input::Infrastructure {

    let mut partial_routes = HashMap::new();
    let mut elementary_routes = Vec::new();
    let mut partial_route_resources :HashMap<usize, HashSet<planner::input::PartialRouteId>> = HashMap::new();
    let mut fresh = { let mut i = 0; move || { i += 1; i } };

    fn convert_routeentryexit(e :&rolling_inf::RouteEntryExit) -> planner::input::SignalId {
        match e {
            rolling_inf::RouteEntryExit::Boundary(_) => planner::input::SignalId::Boundary,
            rolling_inf::RouteEntryExit::Signal(s) => planner::input::SignalId::ExternalId(*s),
            rolling_inf::RouteEntryExit::SignalTrigger { signal, .. } => 
                planner::input::SignalId::ExternalId(*signal),
        }
    }

    let mut boundary_routes :HashMap<rolling_inf::NodeId, HashSet<usize>> = HashMap::new();
    for (route_name,route) in routes.iter() {
        let mut signals = vec![convert_routeentryexit(&route.entry)];
        for i in 0..(route.resources.releases.len()-1) {
            signals.push(planner::input::SignalId::Anonymous(fresh()));
        }
        signals.push(convert_routeentryexit(&route.exit));

        for s in &[&route.entry,&route.exit] {
            if let rolling_inf::RouteEntryExit::Boundary(Some(n)) = s {
                boundary_routes.entry(*n).or_insert(HashSet::new()).insert(*route_name);
            }
        }

        let mut elementary_route = HashSet::new();
        for (i,(entry,exit)) in signals.iter().zip(signals.iter().skip(1)).enumerate() {
            let (length,resources) = if route.resources.releases.len() > 0 {
                let release = route.resources.releases[i].clone();
                (release.length, release.resources)
            } else {
                (route.length, std::iter::empty().collect())
            };

            partial_routes.insert((*route_name, i), planner::input::PartialRoute {
                entry: *entry, exit: *exit,
                conflicts: Vec::new(),
                wait_conflict: None,
                length: length as _,
            });

            for resource in resources.iter() {
                partial_route_resources.entry(*resource)
                    .or_insert(HashSet::new())
                    .insert((*route_name, i));
            }
            elementary_route.insert((*route_name, i));
        }
        elementary_routes.push(elementary_route);
    }

    // second pass adds conflicting routes from resource -> partialroutes map
    for (rn,r) in routes.iter() {
        if r.resources.releases.len() > 0 {
            for (i,rel) in r.resources.releases.iter().enumerate() {
                let mut conflicting_routes = HashSet::new();
                for resource in rel.resources.iter() {
                    if let Some(conflicts) = partial_route_resources.get(resource) {
                        conflicting_routes.extend(conflicts.iter().cloned()
                                                  .filter(|(pr_e,pr_p)| pr_e != rn)
                                                  .map(|pr| (pr,0)));
                    }
                }

                partial_routes.get_mut(&(*rn,i)).unwrap().conflicts =
                    vec![conflicting_routes]; // TODO overlap alternatives
            }
        } else {
            // There are no resources, but we have to add the overlap choice anyway
            partial_routes.get_mut(&(*rn,0)).unwrap().conflicts = 
                vec![std::iter::empty().collect()];
        }
    }

    // Add boundary conflicts
    for (_, set) in boundary_routes {
        //println!("Exlcuding set of routes because they share a boundary: {:?}", set);
        let set :Vec<usize> = set.into_iter().collect();
        for (i,j) in set.iter().flat_map(|x| set.iter().map(move |y| (*x,*y))).filter(|(x,y)| x != y) { 
            let j_choices = partial_routes.get_mut(&(j,0)).unwrap().conflicts.len();
            
            for cs in partial_routes.get_mut(&(i,0)).unwrap().conflicts.iter_mut() {
                for choice in 0..j_choices {
                    cs.insert(((j,0),choice));
                }
            }
        }
    }


    planner::input::Infrastructure { partial_routes, elementary_routes }
}


pub fn convert_plan(il :&Interlocking, 
                    vehicles :&[(usize,Vehicle)], 
                    plan :&PlanSpec) -> Result<planner::input::Usage, ConvertPlanErr> {

    let mut trains = HashMap::new();
    for (t_id,(vehicle_id,visits)) in plan.trains.iter() {
        let vehicle_id = vehicle_id.ok_or(ConvertPlanErr::VehicleRefMissing)?;
        let vehicle = vehicles.iter().find(|(i,v)| *i == vehicle_id).map(|(i,v)| v)
            .ok_or(ConvertPlanErr::VehicleMissing)?;
        let mut planner_visits :Vec<HashSet<usize>> = Vec::new();
        for (visit_i, (visit_id, Visit { locs, dwell})) in visits.iter().enumerate() {
            let mut set = HashSet::new();
            let bdry = if visit_i == 0 { &il.boundary_routes } else { &il.boundary_out_routes };
            for (loc_i, loc) in locs.iter().enumerate() {
                if let Ok(Ref::Node(pt)) = loc {
                    set.extend(bdry.get(pt).into_iter().flat_map(move |rs| rs.iter()));
                }  else {
                    unimplemented!(); // TODO  other types of infrastructure references
                }
            }
            planner_visits.push(set);
        }
        trains.insert(*t_id, planner::input::Train {
            length: vehicle.length,
            visits: planner_visits,
        });
    }

    let mut train_ord = Vec::new();
    for ((train_a,visit_a),(train_b,visit_b), _max_time) in &plan.order {
        // TODO max_time between visits
        let visit_idx = |train_id, visit_id| plan.trains.get(train_id).unwrap()
            .1.iter().position(|(v,_)| v == visit_id).unwrap(); 
        // TODO unwrap crashes if visit_id is missing

        train_ord.push(planner::input::TrainOrd {
            a: (*train_a, visit_idx(*train_a, visit_a)),
            b: (*train_b, visit_idx(*train_b, visit_b)),
        });

    }

    Ok(planner::input::Usage { trains, train_ord })
}




