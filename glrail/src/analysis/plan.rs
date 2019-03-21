use std::collections::{HashMap, HashSet};
use crate::scenario::{Usage, Dispatch, Command, History};
use planner::input::{Problem, RoutePlan};
use rolling::input::staticinfrastructure::{StaticInfrastructure, Routes, Route, Release, RouteEntryExit};
use crate::vehicle::Vehicle;
use crate::model::Derive;

//kk
// TODO: routes contain nodes
// This is not something that rolling cares about, so we 
// should have a map here from routes to nodes which the
// planner can use to convert node-alternatives in  visits 
// to constraints.
//

// convert problem

pub fn convert(vehicles :&[Vehicle], routes :&Routes<usize>, usage :&Usage) -> Problem {
    use planner::input::*;

    // rolling Routes  ->  planner partial routes + elementary routes

    // hs: convertRoutes: resolve_conflicts (join routeparts) ,, splitName routeParts
    //
    


    // first, convert each route to a set fo partial routes
    // then check resource conflict between partial routes

    struct SplitRoute {
        name: (usize,usize),
        entry: Option<usize>,
        exit: Option<usize>,
        length: f64,
        resources: HashSet<usize>, //?
        nodes :HashSet<usize>, //?
    }

    let mut partial_routes = HashMap::new();
    let mut elementary_routes = Vec::new();
    let mut partial_route_resources :HashMap<usize, HashSet<PartialRouteId>> = HashMap::new();
    let mut fresh = { let mut i = 0; move || { i += 1; i } };

    fn convert_routeentryexit(e :&RouteEntryExit) -> SignalId {
        match e {
            RouteEntryExit::Boundary(_) => SignalId::Boundary,
            RouteEntryExit::Signal(s) => SignalId::ExternalId(*s),
            RouteEntryExit::SignalTrigger { signal, .. } => SignalId::ExternalId(*signal),
        }
    }

    for (route_name,route) in routes.iter() {
        let mut signals = vec![convert_routeentryexit(&route.entry)];
        if route.resources.releases.len() > 0 {
            for i in 0..(route.resources.releases.len()-1) { 
                signals.push(SignalId::Anonymous(fresh()));
            }
        }
        signals.push(convert_routeentryexit(&route.exit));

        let mut elementary_route = HashSet::new();
        for (i,(entry,exit)) in signals.iter().zip(signals.iter().skip(1)).enumerate() {

            let (length,resources) = if route.resources.releases.len() > 0 {
                let release = route.resources.releases[i].clone();
                (release.length, release.resources)
            } else {
                (route.length, std::iter::empty().collect())
            };

            partial_routes.insert((*route_name,i), PartialRoute {
                entry: *entry, exit: *exit, 
                conflicts: vec![], // calculated below
                wait_conflict: None, // TODO support overlaps and timeout in route finder
                contains_nodes: std::iter::empty().collect(),
                length: length as _ ,
            });

            for resource in resources.iter() {
                partial_route_resources.entry(*resource)
                    .or_insert(HashSet::new())
                    .insert((*route_name,i));
            }

            elementary_route.insert((*route_name,i));
        }
        elementary_routes.push(elementary_route);
    }

    // second pass adds conflicting routes from resource->partialroute map
    for (rn,r) in routes.iter() {
        if r.resources.releases.len() > 0 {
            for (i,rel) in r.resources.releases.iter().enumerate() {
                let mut conflicting_routes = HashSet::new();
                for resource in rel.resources.iter() {
                    if let Some(conflicts) = partial_route_resources.get(resource) {
                        conflicting_routes.extend(conflicts.iter().cloned()
                                                 // does not conflict with itself
                                                  .filter(|(pr_e,pr_p)| pr_e != rn)
                                                  .map(|pr| (pr,0)));
                    }
                }

                partial_routes.get_mut(&(*rn,i)).unwrap().conflicts = 
                    vec![conflicting_routes]; // TODO overlap alternatives 
            }
        } else {
            // there are no resources. But we have to add the overlap choice anyway.
            partial_routes.get_mut(&(*rn,0)).unwrap().conflicts =
                vec![std::iter::empty().collect()];
        }
    }



    // movement -> train
    let mut trains  = HashMap::new();
    let mut train_ord = Vec::new();

    for (m_i,movement) in usage.movements.iter().enumerate() {
        let vehicle = &vehicles[movement.vehicle_ref];
        let train = Train {
            length: vehicle.length,
            visits: movement.visits.iter().map(|v| {
                v.nodes.iter().cloned().collect() }).collect(),
        };

        trains.insert(m_i, train);
    }

    // TODO timing spec

    Problem { partial_routes, elementary_routes, trains, train_ord }
}


pub fn get_dispatches(vehicles :&[Vehicle], inf :&StaticInfrastructure, routes :&Routes<usize>, usage :&Usage) -> Result<Vec<Dispatch>, String> {
    use planner::input::*;
    use planner::solver::*;

    let problem = convert(vehicles, routes, usage);
    println!("PROBLEM {:#?}", problem);
    let config = Config { n_before: 3, n_after: 3 };

    let routeplan = plan(&config, &problem, |_| true);
    // convert dispatch

    println!("plan() returned {:#?}.", routeplan);

    if let Some(routeplan) = routeplan {
        let commands = convert_dispatch_commands(&routeplan, routes, usage);
        println!("converted to glrail commands: {:#?}", commands);

        // Run simulation on this to get history

        use crate::analysis::sim;
        let history = sim::get_history(vehicles, inf, routes, &commands)?;

        //unimplemented!()
        Ok(vec![Dispatch {
            commands,
            history: Derive::Ok(history),
        }])
    } else {
        Err(format!("No plans found."))
    }
}

pub fn convert_dispatch_commands(routeplan :&RoutePlan, routes :&Routes<usize>, usage :&Usage) -> Vec<(f32,Command)> {
    use std::collections::BTreeSet;
    use crate::scenario::*;
    let mut commands = Vec::new();
    let mut last_active_routes = BTreeSet::new();
    for state in routeplan.iter() {
        let active_routes = state.iter().filter_map(|((elementary,part),train_id)| {
            // use first partial as representative for elementary route
            if *part == 0 && train_id.is_some() {
                Some((*elementary,train_id.unwrap()))
            } else {
                None
            }
        }).collect::<BTreeSet<_>>();

        for (new_route,train_id) in active_routes.difference(&last_active_routes) {
            // check if the route is the birth of a train (comes from boundary)
            match routes[new_route].entry {
                RouteEntryExit::Boundary(_) => {
                    commands.push((0.0, Command::Train(
                                usage.movements[*train_id].vehicle_ref,
                                *new_route)));
                },
                RouteEntryExit::Signal(_) | RouteEntryExit::SignalTrigger { .. } => {
                    commands.push((0.0, Command::Route(*new_route)));
                }
            }
        }

        // TODO barrier?

        last_active_routes = active_routes;
    }

    commands
}
