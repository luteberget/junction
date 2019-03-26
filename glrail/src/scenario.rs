// A scenario is either a dispatch plan or a "movement spec"?
// which results in a set of dispatch plans.
//

use crate::model::*;
use serde::{Serialize, Deserialize};
use rolling::input::staticinfrastructure as rolling_inf;
pub use rolling::output::history::History;
use crate::infrastructure::*;

#[derive(Debug)]
pub enum ScenarioEdit {
    NewDispatch,
    AddDispatchCommand(usize, f32, Command),
    ModifyDispatchCommand(usize, usize, Option<(f32, Command)>),

    NewUsage,
    AddUsageMovement(usize),
    AddUsageMovementVisit(usize, usize),
    SetUsageMovementVehicle(usize, usize, usize),
    SetUsageMovementVisitNodes(usize, usize, usize, Vec<EntityId>),
}


#[derive(Serialize, Deserialize)]
pub enum Scenario {
    Dispatch(Dispatch),
    Usage(Usage, Derive<Vec<Dispatch>>),
}

impl Scenario {
    pub fn set_history(&mut self, h :Derive<History>)  {
        match self {
            Scenario::Dispatch(Dispatch { ref mut history, .. }) => *history = h,
            _ => {},
        }
    }

    pub fn set_usage_dispatches(&mut self, vd :Derive<Vec<Dispatch>>) {
        match self {
            Scenario::Usage(_, ref mut v) => *v = vd,
            _ => {},
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<(f32, Command)>,

    #[serde(skip)]
    pub history :Derive<History>,
}
impl Default for Dispatch {
    fn default() -> Dispatch { Dispatch {
        commands: vec![],
        history: Default::default(),
    }}
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Usage {
    pub movements :Vec<Movement>,
    pub timings :Vec<TimingSpec>,
}

impl Default for Usage {
    fn default() -> Usage { Usage {
        movements: vec![Default::default()],
        timings: vec![],
    }}
}

pub type VisitRef = (usize,usize); // .0 indexes Usage.movements, .1 indexes Movement.visits

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct TimingSpec {
    pub visit_a :VisitRef,
    pub visit_b :VisitRef,
    pub time :Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Movement {
    pub vehicle_ref: usize,
    pub visits: Vec<Visit>,
}


impl Default for Movement {
    fn default() -> Movement { Movement {
        vehicle_ref: 0,
        visits: vec![],
    }}
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Visit {
    pub nodes :Vec<EntityId>,
    pub time :Option<f32>,
}

impl Default for Visit {
    fn default() -> Visit { Visit {
        nodes: vec![],
        time: None,
    }}
}

//#[derive(Serialize, Deserialize)]
//pub struct History {
//    pub moves : Vec<()>,
//}
//impl Default for History {
//    fn default() -> History { History {
//        moves: Vec::new(),
//    }}
//}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Command {
    Route(usize),
    Train(usize,usize),
}


