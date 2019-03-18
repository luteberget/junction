// A scenario is either a dispatch plan or a "movement spec"?
// which results in a set of dispatch plans.
//

use crate::model::*;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum Scenario {
    Dispatch(Dispatch),
    Movement(Movement, Derive<Vec<Dispatch>>),
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<Command>,
    pub history :Derive<History>,
}

#[derive(Serialize, Deserialize)]
pub struct Movement {
    pub visits : (),
}

#[derive(Serialize, Deserialize)]
pub struct History {
    pub moves : Vec<()>,
}

#[derive(Serialize, Deserialize)]
pub struct Command {}
