use crate::infrastructure::*;
use crate::schematic::*;
use crate::scenario::History;
use std::collections::HashMap;

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


fn mk_instant(time :f64, schematic :&Schematic, history: &History) -> Instant {
    use rolling::output::history::*;

    let mut t = 0.0;
    let mut signals : HashMap<ObjectId, SignalAspect> = HashMap::new();
    let mut switches : HashMap<NodeId, SwitchStatus> = HashMap::new();
    let mut sections : HashMap<usize, SectionStatus> = HashMap::new();
    for infevent in &history.inf {
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; if t > time { break; } },
            InfrastructureLogEvent::Authority(sig,l) => {
                match l {
                    Some(_) => signals.insert( unimplemented!() , SignalAspect::Green),
                    None    => signals.insert( unimplemented!() , SignalAspect::Red),
                };
            },
            InfrastructureLogEvent::Reserved(tvd,b) => {
            },
            InfrastructureLogEvent::Occupied(tvd,b) => {
            },
            InfrastructureLogEvent::Position(sw,p) => {
            },
            _ => {}, // TODO route
        }
    }

    let mut geom = Vec::new();
    for (id,aspect) in signals {
        let pt = unimplemented!(); // schematic. ...
        geom.push((DispatchCanvasGeom::SignalAspect(pt,id,aspect), None));
    }

    for (id,pos) in switches {
        //let 
    }

    for (tvd,status) in sections {
        let lines :Vec<Pt>= unimplemented!(); // draw_section(tvd);
        for (p1,p2) in lines.iter().zip(lines.iter().skip(1)) {
            geom.push((DispatchCanvasGeom::SectionStatus(*p1,*p2,status), None));
        }
    }

    Instant { time, geom }
}

