use std::collections::HashMap;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::model::*;
use crate::dgraph::*;

#[derive(Debug)]
pub struct Interlocking {
    pub routes: Vec<(rolling_inf::Route, Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>)>,
    pub boundary_routes: HashMap<Pt, Vec<usize>>,
    pub signal_routes: HashMap<PtA, Vec<usize>>,
}


pub fn calc(dgraph :&DGraph) -> Interlocking {
    let (routes,route_issues) = 
        route_finder::find_routes(Default::default(), &dgraph.rolling_inf)
        .expect("interlocking route finder failed");

    let mut boundary_routes = HashMap::new();
    let mut signal_routes = HashMap::new();
    for (route_idx, (route,_)) in routes.iter().enumerate() {
        match route.entry {
            rolling_inf::RouteEntryExit::Boundary(Some(boundary)) => {
                // Boundary is a NodeId, which should be tied to a Pt in the Dgraph
                if let Some(pt) = dgraph.node_ids.get(&boundary) {
                    boundary_routes.entry(*pt).or_insert(Vec::new()).push(route_idx);
                }
            },
            rolling_inf::RouteEntryExit::Signal(signal) |
            rolling_inf::RouteEntryExit::SignalTrigger { signal , .. } => {
                if let Some(pta) = dgraph.object_ids.get(&signal) {
                    signal_routes.entry(*pta).or_insert(Vec::new()).push(route_idx);
                }
            },
            _ => {},
        }
    }

    let interlocking = Interlocking { routes, boundary_routes, signal_routes };

    interlocking
}
