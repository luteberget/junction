// A scenario is either a dispatch plan or a "movement spec"?
// which results in a set of dispatch plans.
//

use crate::model::*;
use serde::{Serialize, Deserialize};
use rolling::input::staticinfrastructure as rolling_inf;
pub use rolling::output::history::History;
use crate::infrastructure::*;
use crate::graph::*;
use crate::dgraph::DGraph;
use crate::schematic::Schematic;

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
    Usage(Usage, Derive<Vec<Dispatch>>),
}

impl Scenario {
    pub fn set_history(&mut self, h :Derive<HistoryGraph>)  {
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

pub struct HistoryGraph {
    history :History,
    graph: Option<Graph>,
}

impl Default for HistoryGraph {
    fn default() -> HistoryGraph {
        HistoryGraph {
            history: Default::default(),
            graph: None,
        }
    }
}

impl HistoryGraph {
    pub fn new(history :History) -> Self {
        Self { history, graph: None }
    }

    pub fn history(&self) -> &History { &self.history }
    pub fn graph(&mut self, inf :&Infrastructure, dgraph :&DGraph, schematic :&Schematic) -> &Graph {
        if self.graph.is_none() {
            println!("Pre-drawing graph");
            self.graph = Some(Graph::new(0.0, &self.history,inf, dgraph, schematic));
            println!("{:?}", self.graph.as_ref().unwrap());
        }
        self.graph.as_ref().unwrap()
    }
    pub fn set_time(&mut self, t: f32, inf:&Infrastructure, dgraph :&DGraph, schematic :&Schematic) {
        self.graph = Some(Graph::new(t, &self.history, inf, dgraph, schematic));
        println!("new time {} {:?}", t, self.graph.as_ref().unwrap());
        let x = self.graph.as_ref().unwrap().instant.geom.clone();
        for x in x {
            println!("{:?}",x);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dispatch {
    pub commands :Vec<(f32, Command)>,

    #[serde(skip)]
    pub history :Derive<HistoryGraph>,
}
impl Default for Dispatch {
    fn default() -> Dispatch { Dispatch {
        commands: vec![],
        history: Default::default(),
    }}
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct TimingSpec {
    pub visit_a :VisitRef,
    pub visit_b :VisitRef,
    pub time :Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
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

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Command {
    Route(usize),
    Train(usize,usize),
}


