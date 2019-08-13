use std::collections::HashMap;
use crate::viewmodel::*;
use crate::model::*;
use crate::dgraph::*;
use crate::util::VecMap;
use rolling::input::staticinfrastructure as rolling_inf;
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct DispatchView {
    pub history :History,
    pub time_interval :(f32,f32),
    pub pos_interval :(f32,f32),
    pub instant :Instant,
    pub diagram :Diagram, 
}

fn pos_range(diagram :&Diagram) -> (f32,f32) {
    let (mut pmin,mut pmax) = (std::f32::INFINITY, -std::f32::INFINITY);
    for t in &diagram.trains {
        for seg in &t.segments {
            pmin = pmin.min(seg.kms[0] as f32);
            pmin = pmin.min(seg.kms[3] as f32);
            pmax = pmax.max(seg.kms[0] as f32);
            pmax = pmax.max(seg.kms[3] as f32);
        }
    }
    (pmin,pmax)
}

impl DispatchView {
    pub fn from_history(dgraph :&DGraph, history :History) -> DispatchView {
        let t = max_time(&history) as f32;
        let instant = Instant::from(0.0, &history, dgraph);
        let diagram = Diagram::from(&history, dgraph);
        let (pos1,pos2) = pos_range(&diagram);
        let pos_diff = pos2 - pos1;
        //println!("Pos range {:?}", (pos1,pos2));
        DispatchView {
            history: history,
            time_interval: (-0.1*t, 1.1*t),
            pos_interval: (pos1-0.1*pos_diff,pos2+0.1*pos_diff),
            instant: instant,
            diagram: diagram,
        }
    }
}

pub struct InstantCache {
    data :Vec<Option<Instant>>,
}

impl InstantCache {
    pub fn new() -> Self { InstantCache { data: Vec::new() } }
    pub fn dispatch_time(&self, idx :usize) -> Option<f32> {
        self.data.vecmap_get(idx).map(|i| i.time)
    }

    pub fn clear_dispatch(&mut self, idx :usize) {
        self.data.vecmap_remove(idx);
    }

    fn is_cached(&mut self, idx :usize, time :f32) -> bool {
        if let Some(instant) = self.data.vecmap_get(idx) {
            if instant.time == time {
                return true;
            }  
        }
        false
    }
    
    fn update(&mut self, vm :&ViewModel, idx:usize, time :f32) -> Option<()> {
        let dgraph = vm.get_data().dgraph.as_ref()?;
        let dispatch = vm.get_data().dispatch.get(idx)?.as_ref()?;
        self.data.vecmap_insert(idx, Instant::from(time, &dispatch.history, dgraph));
        Some(())
    }

    pub fn get_instant<'a>(&'a mut self, vm :&ViewModel, idx :usize, time :f32) -> Option<&'a Instant> {
        if !self.is_cached(idx,time) { self.update(vm,idx,time); }
        self.data.vecmap_get(idx)
    }
}


#[derive(Debug)]
pub struct Instant {
    pub time :f32,
    pub draw :Vec<Vec<(PtC,PtC)>>,
}

impl Instant {
    pub fn from(time :f32, history :&History, dgraph :&DGraph) -> Instant {
        Instant { time:time, draw: draw_train(time as f64,history,dgraph) }
    }
}


pub fn max_time(h :&History) -> f64 {
    let mut t = 0.0;
    for infevent in &h.inf  {
        use rolling::output::history::*;
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; },
            _ => {}
        }
    }
    t
}

#[derive(Debug)]
pub struct Diagram {
    pub trains: Vec<TrainGraph>,
    pub blocks :Vec<BlockGraph>,
}

impl Diagram {
    pub fn from(history :&History, dgraph :&DGraph) -> Diagram {
        let trains = plot_trains(&history, dgraph);
        let blocks = plot_blocks(&history, dgraph);

        //println!("GOT BLOCKS\n\n{:#?}\n\n", blocks);

        Diagram { trains, blocks }
    }
}

#[derive(Debug)]
pub struct BlockGraph {
    pub pos :(f64,f64),
    pub reserved :(f64,f64),
    pub occupied :(f64,f64),
    pub train :usize,
    pub info: String,
}

#[derive(Debug)]
pub struct TrainGraph {
    pub segments :Vec<TrainGraphSegment>,
}

#[derive(Debug)]
pub struct TrainGraphSegment {
    pub start_time :f64,
    pub kms :[f64;4],
    pub start_vel :f64,
    pub dt: f64,
    pub acc :f64,
}

fn plot_blocks(history :&History, dgraph :&DGraph) -> Vec<BlockGraph> {
    let mut output = Vec::new();

    // TVD object id -> time interval occupied
    // integrate over trains:
    //   if visiting node which enters TVD inside time interval (minus tolerance), 
    //   add node as entry point for tvd visit.

    use rolling::input::staticinfrastructure::*;
    use rolling::output::history::*;
    let mut t = 0.0;
    let mut reserved : HashMap<ObjectId,f64> = HashMap::new(); // Reserved at time
    let mut occupied : HashMap<ObjectId,(f64,(f64,NodeId))> = HashMap::new(); // Became occupied at time
    let mut vacant : HashMap<ObjectId,(f64,(f64,NodeId),(f64,NodeId))> = HashMap::new(); // Became occupied at time
    for infevent in &history.inf {
        //println!("infevent {:?}", infevent);
        //println!("rserved {:?}", reserved);
        //println!("occupied {:?}", occupied);
        //println!("vacant {:?}", vacant);
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; },
            InfrastructureLogEvent::Reserved(tvd,on) if *on => { reserved.insert(*tvd, t); }
            InfrastructureLogEvent::Occupied(tvd,on,node,train) if *on => {
                if let Some(reserved_t) = reserved.remove(tvd) {
                    occupied.insert(*tvd, (reserved_t, (t, *node)));
                }
            },
            InfrastructureLogEvent::Occupied(tvd,on,node,train) if !*on => {
                if let Some((reserved_t,(occupied_t,occ_node))) = occupied.remove(tvd) {
                    vacant.insert(*tvd, (reserved_t, (occupied_t,occ_node),(t, *node)));
                }
            },
            InfrastructureLogEvent::Reserved(tvd,on) if !*on => { 
                if let Some((res_t, (occ_t, occ_node), (vac_t, vac_node))) = vacant.remove(tvd) {
                    if let Some(pos1) = dgraph.mileage.get(&occ_node) {
                        if let Some(pos2) = dgraph.mileage.get(&vac_node) {
                            output.push(BlockGraph {
                                pos: (pos1.min(*pos2), pos1.max(*pos2)),
                                reserved: (res_t, t),
                                occupied: (occ_t, vac_t),
                                train: 0, // TODO
                                info: format!("info"), // TODO
                            });
                        }
                    }
                }
            }
            _ => {},
        }
    }

    output
}


fn plot_trains(history :&History, dgraph :&DGraph) -> Vec<TrainGraph> {
    let mut output = Vec::new();
    for (train_i, (name, params, events)) in history.trains.iter().enumerate() {
        let mut segments =  Vec::new();
        use rolling::railway::dynamics::*;
        use rolling::output::history::*;
        let mut edge_x = 0.0;
        let mut t = 0.0;
        let mut current_edge_pos = None;
        let mut prev_v = 0.0; // TODO allow train v0 != 0
        for e in events {
            match e {
                //TODO sight?
                TrainLogEvent::Wait(dt) => { t += dt; },
                TrainLogEvent::Edge(a,b) => {
                    edge_x = 0.0;
                    if let Some(b) = b {
                        current_edge_pos = Some((*dgraph.mileage.get(a).unwrap(),
                                            *dgraph.mileage.get(b).unwrap(),
                                            edge_length(&dgraph.rolling_inf, *a, *b).unwrap()));
                    } else {
                        let sign = segments.last().map(|s:&TrainGraphSegment| 
                                (s.kms[3] - s.kms[0]).signum()).unwrap_or(1.);
                        current_edge_pos = Some((*dgraph.mileage.get(a).unwrap(),
                                            dgraph.mileage.get(a).unwrap() + sign*1000.0,
                                            1000.0));
                    }
                },
                TrainLogEvent::Move(dt,action,DistanceVelocity { dx, v }) => {
                    let acc = if *dt > 0.0 { (*v - prev_v)/dt } else { 0.0 };
                    if let Some((pos1,pos2,edge_length)) = current_edge_pos {

                        let mut kms = [0.;4];
                        let (mut sample_x, mut sample_v) = (edge_x, prev_v);
                        for i in 0..=3 {
                            kms[i] = glm::lerp_scalar(pos1, pos2, sample_x/edge_length);
                            let dt = dt / 3.0;
                            sample_x += sample_v * dt + 0.5 * acc * dt * dt;
                            sample_v += acc * dt;
                        }

                        segments.push(TrainGraphSegment { 
                            start_time: t,
                            start_vel: prev_v,
                            dt: *dt,
                            kms: kms,
                            acc: acc });
                    }
                    t += dt;
                    prev_v = *v;
                    edge_x += dx;
                },
                _ => {},
            }
        }
        output.push(TrainGraph { segments });
    }
    output
}


pub fn draw_train(time :f64, history :&History, dgraph :&DGraph) -> Vec<Vec<(PtC,PtC)>> {
    let mut trains = Vec::new();
    for (train_i, (name, params, events)) in history.trains.iter().enumerate() {

        use rolling::railway::dynamics::*;
        use rolling::output::history::*;
        let mut t = 0.0;
        let mut edges = Vec::new();
        let mut velocity = 0.0;

        let mut lines = Vec::new();
        for e in events {
            match e {
                TrainLogEvent::Edge(a,b) => { edges.push(((*a,*b), 0.0, 0.0)); },
                TrainLogEvent::Move(dt, action, DistanceVelocity { dx, v }) => {
                    let update_x = if t + *dt < time { *dx } else {
                        dynamic_update(params, velocity, DriverPlan { action: *action, dt: time - t}).dx };
                    edges.last_mut().unwrap().2 += update_x;
                    truncate_edge_list(&mut edges, params.length);
                    velocity = *v;
                    t += *dt;
                },
                TrainLogEvent::Wait(dt) => { t += dt; },
                //TrainLogEvent::Sight(id, value) => {
                //},
                _ => {},
            }

            if t >= time { break; }
        }

        for e in edges {
            if let Some(line) = draw_edge_with_offset(dgraph, e) {
                lines.extend(line);
            }
        }

        trains.push(lines);
    }
    trains
}

pub fn draw_edge_with_offset(dgraph :&DGraph, (e,offset1,offset2) :((rolling_inf::NodeId, Option<rolling_inf::NodeId>), f64, f64)) -> Option<Vec<(PtC,PtC)>> {
    let (a,b) = (e.0, e.1?);
    let vec = dgraph.edge_lines.get(&(a,b))?;
    let edge_length = edge_length(&dgraph.rolling_inf, a,b)?;
    let line_length = pline_length(vec);
    let section = pline_section(vec, 
                                (offset1 as f32) / (edge_length as f32) *(line_length as f32), 
                                (offset2 as f32) / (edge_length as f32) *(line_length as f32));
    Some(section)
}

pub fn pline_length(v :&Vec<PtC>) -> f32 {
    v.iter().zip(v.iter().skip(1)).map(|(a,b)| glm::length(&(b-a))).sum()
}

pub fn line_section((p1,p2) :(&PtC,&PtC), a :f32, b :f32) -> Option<(PtC,PtC)>{
    let len = glm::length(&(p2-p1));
    if a > len || b < 0.0 { return None; }
    let pa = if a < 0.0 { *p1 } else { glm::lerp(p1,p2, a / len) };
    let pb = if b > len { *p2 } else { glm::lerp(p1,p2, b / len) };
    Some((pa,pb))
}

pub fn pline_section(p :&Vec<PtC>, a :f32, b :f32) -> Vec<(PtC,PtC)> {
    let mut output = Vec::new();
    let mut t = 0.0;
    for (p1,p2) in p.iter().zip(p.iter().skip(1)) {
        if let Some(l) = line_section((p1,p2), a - t, b - t) {
            output.push(l);
        }
        t += glm::length(&(p2-p1));
    }
    output
}


fn truncate_edge_list(e :&mut Vec<((usize, Option<usize>), f64, f64)>, mut l :f64) {
    let mut del = false;
    for i in (0..e.len()).rev() {
        if del {
            e.remove(i);
        } else {
            let (_, ref mut a, ref mut b) = e[i];
            if *b - *a > l {
                *a = *b - l;
                del = true;
            } else {
                l -= *b - *a;
            }
        }
    }
}


