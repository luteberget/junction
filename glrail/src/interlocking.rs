use crate::model::*;
use crate::infrastructure::*;
use serde::{Serialize, Deserialize};

pub use rolling::input::staticinfrastructure::Route;

#[derive(Serialize, Deserialize)]
pub struct Interlocking {
    pub derive :Option<DeriveInterlocking>,

    #[serde(skip)]
    pub routes :Derive<Vec<Route>>,
}

impl Interlocking {
    pub fn new_default() -> Self {
        Interlocking {
            derive :Some(DeriveInterlocking::new_default()),
            routes :Derive::Ok(Vec::new()),
        }
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


