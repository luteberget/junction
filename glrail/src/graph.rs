use crate::infrastructure::*;
use crate::schematic::*;
use crate::scenario::History;
use crate::dgraph::*;
use crate::model::*;
use rolling::input::staticinfrastructure as rolling_inf;
use bimap::BiMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Graph {
    pub time_range :f64,
    pub instant :Instant,
    pub trains :Vec<TrainGraph>,
}

impl Graph {
    pub fn new(t :f32, history :&History, inf :&Infrastructure, dgraph: &DGraph, schematic: &Schematic) -> Self {
        let time = t as f64;
        let max_t = max_time(history);
        let instant = mk_instant(time, history, inf, schematic, dgraph);
        return Graph {
            time_range: max_t,
            instant: instant,
            trains: Vec::new()
        };
    }
}

#[derive(Debug)]
pub struct TrainGraph {

}



#[derive(Clone)]
#[derive(Debug)]
pub struct Instant {
    pub time :f64,
    pub geom: Vec<(DispatchCanvasGeom, Option<InfoPointer>)>,
}

#[derive(Clone,Copy)]
#[derive(Debug)]
pub enum InfoPointer {
    Train(usize),
}

#[derive(Clone,Copy)]
#[derive(Debug)]
pub enum SignalAspect { Red, Green }
#[derive(Clone,Copy)]
#[derive(Debug)]
pub enum SectionStatus { Free, Reserved, Occupied, Overlap }
#[derive(Clone,Copy)]
#[derive(Debug)]
pub enum SwitchStatus { ControlledLeft, ControlledRight, Uncontrolled }

#[derive(Clone,Copy)]
#[derive(Debug)]
pub enum DispatchCanvasGeom {
    SignalAspect(Pt,ObjectId,SignalAspect), // location, signalid, red-green for now
    SectionStatus(Pt,Pt,SectionStatus),
    SwitchStatus(Pt, NodeId, SwitchStatus),
    TrainLoc(Pt,Pt,usize),
}


fn max_time(history :&History) -> f64 {

    // TODO there seems to be a bug in rolling when
    // trains are intersection (un-physical)
    let mut t = 0.0;
    for infevent in &history.inf {
        use rolling::output::history::*;
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; },
            _ => {},
        }
    }

    t
}

fn mk_instant( time :f64, history: &History, 
               inf: &Infrastructure, schematic :&Schematic, dgraph :&DGraph) 
    -> Instant {
    use rolling::output::history::*;
    let object_ids = &dgraph.object_ids;
    let mut t = 0.0;
    let mut signals : HashMap<ObjectId, SignalAspect> = HashMap::new();
    let mut switches : HashMap<NodeId, SwitchStatus> = HashMap::new();
    let mut sections_reserved : HashMap<usize, bool> = HashMap::new();
    let mut sections_occupied : HashMap<usize, bool> = HashMap::new();
    for infevent in &history.inf {
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; if t > time { break; } },
            InfrastructureLogEvent::Authority(sig_d,l) => {
                // sig has type rolling_inf::ObjectId
                if let Some(EntityId::Object(sig_g)) = dgraph.object_ids.get_by_right(sig_d) {
                    match l {
                        Some(_) => signals.insert(*sig_g, SignalAspect::Green),
                        None    => signals.insert(*sig_g, SignalAspect::Red),
                    };
                }
            },
            InfrastructureLogEvent::Reserved(tvd,b) => {
                sections_reserved.insert(*tvd,*b);
            },
            InfrastructureLogEvent::Occupied(tvd,b) => {
                sections_occupied.insert(*tvd,*b);
            },
            InfrastructureLogEvent::Position(sw,p) => {
            },
            _ => {}, // TODO route
        }
    }

    let mut sections :HashMap<usize, SectionStatus> = sections_reserved.into_iter().map(|(k,v)| {
        (k, if v { SectionStatus::Reserved } else { SectionStatus::Free })}).collect();
    for (sec,occ) in sections_occupied {
        if occ {
            sections.insert(sec, SectionStatus::Occupied);
        }
    }

    let mut geom = Vec::new();
    for (id,aspect) in signals {
        // get object info
        // TODO if schematic plan had proper object coordinates, this step could be skipped
        let Object(tr,p,obj) = inf.get_object(&id).unwrap();
        if let Some((pt,tangent)) = schematic.track_line_at(tr,*p) {
            geom.push((DispatchCanvasGeom::SignalAspect(pt,id,aspect), None));
        }
    }

    for (id,pos) in switches {
        //let 
    }

    for (tvd,status) in sections {
        // tvd has type rolling_inf::ObjectId
        // which is the key of DGraph.tvd_sections
        if let Some(edges) = dgraph.tvd_sections.get(&tvd) {
            // edges is Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>.
            // can be looked up in DGraph.edge_intervals
            for edge in edges.iter() {
                if let Some(Interval { track, p1, p2 }) = dgraph.edge_intervals.get(edge) {
                    // TODO better line handling, this fails at bends.
                    if let Some((pt1,_)) = schematic.track_line_at(track, *p1) {
                        if let Some((pt2,_)) = schematic.track_line_at(track, *p2) {
                            geom.push((DispatchCanvasGeom::SectionStatus(pt1,pt2,status.clone()), None));
                        }
                    }
                }
            }
        }
    }

    Instant { time, geom }
}

