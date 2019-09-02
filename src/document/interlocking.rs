use std::collections::HashMap;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::document::model::*;
use crate::document::dgraph::*;

#[derive(Debug)]
pub struct Interlocking {
    pub routes: Vec<RouteInfo>,
    pub boundary_routes: HashMap<Pt, Vec<usize>>,
    pub boundary_out_routes: HashMap<Pt, Vec<usize>>,
    pub signal_routes: HashMap<PtA, Vec<usize>>,
    pub alternatives :HashMap<(Ref,Ref), Vec<usize>>,
}

impl Interlocking {
    pub fn get_routes(&self, thing :Ref) -> Option<&Vec<usize>> {
        match thing {
            Ref::Node(pt) => self.boundary_routes.get(&pt),
            Ref::Object(pta) => self.signal_routes.get(&pta),
            _ => None,
        }
    }

    pub fn find_route(&self, spec :&RouteSpec) -> Option<&usize> {
        let alternatives = self.alternatives.get(&(spec.from,spec.to))?;
        alternatives.get(spec.alternative.min(alternatives.len()))
    }
}


#[derive(Debug)]
pub struct RouteInfo {
    pub route :rolling_inf::Route,
    pub id :RouteSpec,
    pub path :Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>,
}

impl RouteInfo {
    pub fn start_mileage(&self, dgraph :&DGraph) -> Option<f64> {
        self.path.get(0).and_then(|(n,_)| dgraph.mileage.get(n)).cloned()
    }
}


pub fn calc(dgraph :&DGraph) -> Interlocking {
    let (routes,route_issues) = 
        route_finder::find_routes(Default::default(), &dgraph.rolling_inf)
        .expect("interlocking route finder failed");

    let mut boundary_routes = HashMap::new();
    let mut boundary_out_routes = HashMap::new();
    let mut signal_routes = HashMap::new();
    let mut route_info = Vec::new();
    let mut alternatives : HashMap<(Ref,Ref), Vec<usize>> = HashMap::new();
    for (route_idx, (route,path)) in routes.into_iter().enumerate() {
        let from = match route.entry {
            rolling_inf::RouteEntryExit::Boundary(Some(boundary)) => {
                // Boundary is a NodeId, which should be tied to a Pt in the Dgraph
                if let Some(pt) = dgraph.node_ids.get(&boundary) {
                    boundary_routes.entry(*pt).or_insert(Vec::new()).push(route_idx);
                }

                Ref::Node(dgraph.node_ids[&boundary])
            },
            rolling_inf::RouteEntryExit::Signal(signal) |
            rolling_inf::RouteEntryExit::SignalTrigger { signal , .. } => {
                if let Some(pta) = dgraph.object_ids.get(&signal) {
                    signal_routes.entry(*pta).or_insert(Vec::new()).push(route_idx);
                }

                Ref::Object(dgraph.object_ids[&signal])
            },
            _ => panic!(), // TODO is Boundary(None)  relevant?
        };

        let to = match route.exit {
            rolling_inf::RouteEntryExit::Boundary(Some(boundary)) => {
                if let Some(pt) = dgraph.node_ids.get(&boundary) {
                    boundary_out_routes.entry(*pt).or_insert(Vec::new()).push(route_idx);
                }
                Ref::Node(dgraph.node_ids[&boundary])
            },
            rolling_inf::RouteEntryExit::Signal(signal) |
            rolling_inf::RouteEntryExit::SignalTrigger { signal , .. } => {
                Ref::Object(dgraph.object_ids[&signal])
            },
            _ => panic!(), // TODO is Boundary(None)  relevant?
        };

        let alternative_vec = alternatives.entry((from,to))
            .or_insert(Vec::new());
        alternative_vec.push(route_idx);
        let alternative = alternative_vec.len()-1;

        route_info.push(RouteInfo { route, id: RouteSpec { from, to, alternative }, path });
    }


    let interlocking = Interlocking { routes: route_info, 
        boundary_routes, boundary_out_routes, signal_routes, alternatives };

    interlocking
}
