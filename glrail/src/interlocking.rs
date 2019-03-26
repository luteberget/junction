use crate::model::*;
use crate::dgraph::*;
use crate::infrastructure::*;
use serde::{Serialize, Deserialize};

pub use rolling::input::staticinfrastructure::Route;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Interlocking {
    pub derive :Option<DeriveInterlocking>,

    #[serde(skip)]
    pub routes :Derive<Arc<Vec<Route>>>,
}

impl Interlocking {
    pub fn new_default() -> Self {
        Interlocking {
            derive :Some(DeriveInterlocking::new_default()),
            routes :Derive::Ok(Arc::new(Vec::new())),
        }
    }

    pub fn routes_from_signal<'a>(&'a self, dgraph :&'a DGraph, 
                                entity: EntityId) -> Box<Iterator<Item = &'a Route> + 'a> {
        if let Some(RollingId::StaticObject(id)) = dgraph.entity_names.get_by_right(&entity) {
            if let Some(routes) = self.routes.get() {
                use rolling::input::staticinfrastructure::{Route, RouteEntryExit};
                return Box::new(routes.iter().filter(move |r| r.entry == RouteEntryExit::Signal(*id)))
            }
        }
        Box::new(std::iter::empty())
    }

    pub fn routes_from_boundary<'a>(&'a self, dgraph :&'a DGraph, 
                                entity: EntityId) -> Box<Iterator<Item = &'a Route> + 'a> {
        if let Some(RollingId::Node(id)) = dgraph.entity_names.get_by_right(&entity) {
            if let Some(routes) = self.routes.get() {
                use rolling::input::staticinfrastructure::{Route, RouteEntryExit};
                return Box::new(routes.iter().filter(move |r| r.entry == RouteEntryExit::Boundary(Some(*id))))
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


