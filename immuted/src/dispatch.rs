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
    pub diagram :Vec<TrainGraph>,
}

impl DispatchView {
    pub fn from_history(dgraph :&DGraph, history :History) -> DispatchView {
        let t = max_time(&history) as f32;
        let x = max_pos(&history) as f32;
        let instant = Instant::from(0.0, &history, dgraph);
        let diagram = plot_trains(&history);
        DispatchView {
            history: history,
            time_interval: (-0.1*t, 1.1*t),
            pos_interval: (-0.1*x, 1.1*x),
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

pub fn max_pos(h :&History) -> f64 {
    let mut x :f64 = 0.0;
    for TrainGraph { segments } in plot_trains(h) {
        for seg in segments {
            x = x.max(seg.start_pos)
                .max(seg.start_pos + seg.dt*seg.start_vel + 0.5*seg.acc*seg.dt*seg.dt);
        }
    }
    x
}

#[derive(Debug)]
pub struct TrainGraph {
    pub segments :Vec<TrainGraphSegment>,
}

#[derive(Debug)]
pub struct TrainGraphSegment {
    pub start_time :f64,
    pub start_pos :f64,
    pub start_vel :f64,
    pub dt: f64,
    pub acc :f64,
}

fn plot_trains(history :&History) -> Vec<TrainGraph> {
    let mut output = Vec::new();
    for (train_i, (name, params, events)) in history.trains.iter().enumerate() {
        let mut segments =  Vec::new();
        use rolling::railway::dynamics::*;
        use rolling::output::history::*;
        let mut t = 0.0;
        let mut x = 0.0;
        let mut prev_v = 0.0; // TODO allow train v0 != 0
        for e in events {
            match e {
                //TODO sight?
                TrainLogEvent::Wait(dt) => { t += dt; },
                TrainLogEvent::Move(dt,action, DistanceVelocity {dx, v }) => {
                    let acc = if *dt > 0.0 { (*v - prev_v)/dt } else { 0.0 };
                    segments.push(TrainGraphSegment { 
                        start_time: t,
                        start_pos: x,
                        start_vel: prev_v,
                        dt: *dt,
                        acc: acc });
                    t += dt;
                    x += dx;
                    prev_v = *v;
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


