use std::collections::HashSet;
use crate::document::interlocking::*;
use crate::document::dgraph::*;
use crate::synthesis::*;
use rolling::input::staticinfrastructure as rolling_inf;

/// A multiplan assigns a set of abstract dispatches
/// to each usage.
pub type MultiPlan = Vec<(usize, Vec<AbstractDispatch>)>; 
pub type AbstractDispatch = Vec<AbstractCommand>;

pub struct AbstractCommand {
    pub from :Result<Pt,PtA>,
    pub to :Result<Pt,PtA>,
    pub switches :HashSet<(Pt, rolling_inf::SwitchPosition)>,
    pub train :usize,
}


//TODO didn't we need BiMap in Dgraph after all?

fn abstract_dispatches(
    dgraph :&DGraph,
    il :&Interlocking,
    routeplan :&planner::input::RoutePlan
    ) -> Vec<AbstractCommand> {

    let mut output = Vec::new();
    let mut last_active_routes = BTreeSet::new();
    for state in routeplan.iter() {
        let active_routes = state.iter().filter_map(|((el,part), train_id)| {
            if *part == 0 && train_id.is_some() {
                Some((*el, train_id.unwrap()))
            } else {
                None
            }
        }).collect::<BTreeSet<_>>();

        let mut trains : HashMap<usize, Vec<usize>> = HashMap::new(); // Train -> Vec<ElementaryRoyute>
        for (new_route, train_id) in active_routes.difference(&last_active_routes) {
            trains.entry(*train_id).or_insert(Vec::new()).push(*new_route);
        }

        for (train_id, route_ids) in trains {
            let mut entries :HashSet<_> = route_ids.iter()
                .map(|rid| ignore_trigger(il.routes[*rid].route.entry)).collect();
            let mut exits   :HashSet<_> = route_ids.iter()
                .map(|rid| ignore_trigger(il.routes[*rid].route.exit)).collect();
            let mut switches :HashSet<(_,_)> = HashSet::new();

            for rid in route_ids {
                let route = &il.routes[rid].route;
                entries.remove(&ignore_trigger(route.exit));
                exits.remove(&ignore_trigger(route.entry));
                for (sw,side) in route.resources.switch_positions.iter() {
                    switches.insert((*dgraph.object_ids.get_by_left(sw).unwrap(), *side));
                }
            }
            assert_eq!(entries.len(), 1); assert_eq!(exits.len(), 1); 

            output.push(AbstractCommand { 
                from: convert_routeentryexit(dgraph, *entries.iter().nth(0).unwrap()),
                to: convert_routeentryexit(dgraph, *exits.iter().nth(0).unwrap()),
                switches: switches,
                train: train_id,
            });
        }

        last_active_routes = active_routes;
    }

    output
}

pub fn concrete_dispatch(dgraph :&DGraph, il :&Interlocking, ad :&AbstractCommand) -> Vec<usize> {
    let mut curr_start = ad.from;
    let mut end = ad.to;
    let mut output = Vec::new();

    'ds: while curr_start != end {
        let route_idxs = match curr_start {
            Ok(nd) => il.boundary_routes[&nd].iter(),
            Err(pt) => il.signal_routes[&pt].iter(),
        };

        'rs: for route_idx in route_idxs {
            let route = &il.routes[*route_idx].route;
            let switches = route.resources.switch_positions.iter()
                .map(|(sw,side)| (*dgraph.object_ids.get_by_left(sw).unwrap(), *side))
                .collect::<HashSet<(_,_)>>();

            let sw_ok = switches.difference(&ad.switches).nth(0).is_none();
            if sw_ok {
                output.push(*route_idx);
                curr_start = convert_routeentryexit(dgraph, route.exit);
                continue 'ds;
            } else {
                continue 'rs;
            }
        }
        panic!()
    }

    output
}

fn ignore_trigger(r :rolling_inf::RouteEntryExit) -> rolling_inf::RouteEntryExit {
    use self::rolling_inf::RouteEntryExit;
    match r {
        RouteEntryExit::SignalTrigger { signal, .. } => RouteEntryExit::Signal(signal),
        x => x,
    }
}

fn convert_routeentryexit(dgraph :&DGraph, x :rolling_inf::RouteEntryExit) -> Result<Pt,PtA> {
    match x {
        rolling_inf::RouteEntryExit::Boundary(Some(nd)) => 
            Ok(*dgraph.node_ids.get_by_left(&nd).unwrap()),
        rolling_inf::RouteEntryExit::Signal(signal) |
        rolling_inf::RouteEntryExit::SignalTrigger { signal, .. } => 
            Err(*dgraph.object_ids.get_by_left(&signal).unwrap()),
        _ => panic!(),
    }
}
