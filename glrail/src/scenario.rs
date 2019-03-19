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

impl Scenario {
    pub fn set_history(&mut self, h :Derive<History>)  {
        match self {
            Scenario::Dispatch(Dispatch { ref mut history, .. }) => *history = h,
            _ => {},
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<(f32, Command)>,
    pub history :Derive<History>,
}
impl Default for Dispatch {
    fn default() -> Dispatch { Dispatch {
        commands: vec![
            (10.553151, Command::Train(5,5)),
            (53.12, Command::Route(9)),
            (53.19, Command::Route(8))],
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
pub enum Command {
    Route(usize),
    Train(usize,usize),
}


