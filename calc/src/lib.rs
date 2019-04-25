use junc_model::*;
use junc_model::infrastructure::Infrastructure;
use junc_model::interlocking::*;

use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;

use rolling::output::history::History;

pub mod analysis;

pub mod schematic;
pub mod dgraph;
pub mod dataset;

use schematic::*;
use dgraph::*;
use dataset::*;

pub enum Derive<T> {
    Wait,
    ShortWait(Instant, T),
    Ok(T),
    Err(String),
}

impl<T :Default> Default for Derive<T> {
    fn default() -> Self {
        Derive::Ok(Default::default())
    }
}

impl<T> Derive<T> {
    pub fn get(&self) -> Option<&T> {
        if let Derive::Ok(val) = self { Some(val) } else { None }
    }
}

pub struct DerivedModel {
    pub inf :Derive<Arc<Infrastructure>>,
    pub schematic :Derive<Schematic>, // only used by GUI thread
    pub dgraph: Derive<Arc<DGraph>>,
    pub interlocking: Derive<Arc<Interlocking>>,
    pub custom_datasets: HashMap<usize, Derive<DataSet>>, // only used by GUI thread?
    pub dispatch: HashMap<usize, Derive<History>>, // only used by GUI thread?
}

impl DerivedModel {
    pub fn new(model :&junc_model::Model) -> DerivedModel {
        let m = DerivedModel {
            inf: Derive::Wait,
            schematic: Derive::Wait,
            dgraph: Derive::Wait,
            interlocking: Derive::Wait,
            custom_datasets: model.custom_datasets.iter().enumerate()
                .map(|(i,_)| (i, Derive::Wait)).collect(),
            dispatch: model.scenarios.iter().enumerate()
                .map(|(i,_)| (i, Derive::Wait)).collect(),
        };

        m
    }
}
