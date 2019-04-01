use crate::model::*;
use crate::dgraph::*;
use crate::infrastructure::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub use rolling::input::staticinfrastructure::Route;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Interlocking {
    pub derive :Option<DeriveInterlocking>,

    #[serde(skip)]
    pub routes :Derive<Arc<(Vec<Route>, HashMap<EntityId, Vec<usize>>)>>,
}

impl Interlocking {
    pub fn new_default() -> Self {
        Interlocking {
            derive :Some(DeriveInterlocking::new_default()),
            routes :Derive::Ok(Arc::new((Vec::new(), HashMap::new()))),
        }
    }

    pub fn routes_from_signal<'a>(&'a self, dgraph :&'a DGraph, 
                                object_id: ObjectId) -> Box<Iterator<Item = (usize, &'a Route)> + 'a> {
        if let Some(id) = dgraph.object_ids.get_by_left(&EntityId::Object(object_id)) {
            if let Some(arc) = self.routes.get().clone() {
                let routes = &arc.0;
                use rolling::input::staticinfrastructure::{Route, RouteEntryExit};
                return Box::new(routes.iter().enumerate().filter(move |(_,r)| r.entry == RouteEntryExit::Signal(*id)))
            }
        }
        Box::new(std::iter::empty())
    }

    pub fn routes_from_boundary<'a>(&'a self, dgraph :&'a DGraph, 
                                node_id: NodeId) -> Box<Iterator<Item = (usize, &'a Route)> + 'a> {
        if let Some(id) = dgraph.node_ids.get_by_left(&EntityId::Node(node_id)) {
            if let Some(arc) = self.routes.get() {
                let routes = &arc.0;
                use rolling::input::staticinfrastructure::{Route, RouteEntryExit};
                return Box::new(routes.iter().enumerate().filter(move |(_,r)| r.entry == RouteEntryExit::Boundary(Some(*id))))
            }
        }
        Box::new(std::iter::empty())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeriveInterlocking {
}

impl DeriveInterlocking {
    pub fn new_default() -> Self {
        DeriveInterlocking {}
    }
}


