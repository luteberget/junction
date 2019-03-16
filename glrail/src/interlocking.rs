use crate::model::*;
use crate::infrastructure::*;

pub struct Interlocking {
    derive :Option<DeriveInterlocking>,
    routes :Derive<Vec<Route>>,
}

impl Interlocking {
    pub fn new_default() -> Self {
        Interlocking {
            derive :Some(DeriveInterlocking::new_default()),
            routes :Derive::Ok(Vec::new()),
        }
    }
}

pub struct DeriveInterlocking {
}

impl DeriveInterlocking {
    pub fn new_default() -> Self {
        DeriveInterlocking {}
    }
}

pub struct Route {
    pub start :usize,
    pub end :usize,
}

