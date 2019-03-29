use crate::infrastructure::*;
use crate::schematic::*;

pub struct Instant {
    pub time :f64,
    pub geom: Vec<(DispatchCanvasGeom, Option<InfoPointer>)>,
}

pub enum InfoPointer {
    Train(usize),
}

pub enum SignalAspect { Red, Green }
pub enum SectionStatus { Free, Reserved, Occupied, Overlap }
pub enum SwitchStatus { ControlledLeft, ControlledRight, Uncontrolled }

pub enum DispatchCanvasGeom {
    SignalAspect(Pt,ObjectId,SignalAspect), // location, signalid, red-green for now
    SectionStatus(Pt,Pt,SectionStatus),
    SwitchStatus(Pt, NodeId, SwitchStatus),
    TrainLoc(Pt,Pt,usize),
}


