// A scenario is either a dispatch plan or a "movement spec"?
// which results in a set of dispatch plans.
//

use crate::*;
use serde::{Serialize, Deserialize};
pub use rolling::output::history::History;

#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub struct DispatchId(usize);

#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub struct UsageId(usize);

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
    AddUsageTimingSpec(usize),
    SetUsageTimingSpec(usize,usize,usize,usize,usize,usize,Option<f32>),
}


#[derive(Serialize, Deserialize)]
pub enum Scenario {
    Dispatch(Dispatch),
    Usage(Usage),
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<(f32, Command)>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct Usage {
    pub movements :Vec<Movement>,
    pub timings :Vec<TimingSpec>,
}

pub type VisitRef = (usize,usize); // .0 indexes Usage.movements, .1 indexes Movement.visits

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct TimingSpec {
    pub visit_a :VisitRef,
    pub visit_b :VisitRef,
    pub time :Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct Movement {
    pub vehicle_ref: usize,
    pub visits: Vec<Visit>,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone)]
pub struct Visit {
    pub nodes :Vec<EntityId>,
    pub time :Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Command {
    Route(usize),
    Train(usize,usize),
}


