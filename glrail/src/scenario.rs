// A scenario is either a dispatch plan or a "movement spec"?
// which results in a set of dispatch plans.
//

use crate::model::*;
use serde::{Serialize, Deserialize};

pub enum ScenarioEdit {
    NewDispatch,
    NewMovement,
    AddCommand(usize, f32, Command),
    ModifyCommand(usize, usize, Option<(f32, Command)>),
}


#[derive(Serialize, Deserialize)]
pub enum Scenario {
    Dispatch(Dispatch),
    Movement(Movement, Derive<Vec<Dispatch>>),
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<(f32, Command)>,
    pub history :Derive<History>,
}
impl Default for Dispatch {
    fn default() -> Dispatch { Dispatch {
        commands: Vec::new(),
        history: Default::default(),
    }}
}

#[derive(Serialize, Deserialize)]
pub struct Movement {
    pub visits : (),
}
impl Default for Movement {
    fn default() -> Movement { Movement {
        visits: (),
    }}
}

#[derive(Serialize, Deserialize)]
pub struct History {
    pub moves : Vec<()>,
}
impl Default for History {
    fn default() -> History { History {
        moves: Vec::new(),
    }}
}

#[derive(Serialize, Deserialize)]
pub struct Command {}
