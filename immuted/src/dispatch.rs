use std::collections::{HashMap, HashSet};
use crate::viewmodel::*;
use crate::model::*;
use crate::objects::*;
use crate::dgraph::*;
use crate::util::VecMap;
use rolling::input::staticinfrastructure as rolling_inf;
use rolling_inf::{ObjectId};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct DispatchView {
    pub history :History,
    pub time_interval :(f32,f32),
    pub max_t :f32,
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
            pmin = pmin.min(seg.end_kms[0] as f32);
            pmin = pmin.min(seg.end_kms[3] as f32);
            pmax = pmax.max(seg.kms[0] as f32);
            pmax = pmax.max(seg.kms[3] as f32);
            pmax = pmax.max(seg.end_kms[0] as f32);
            pmax = pmax.max(seg.end_kms[3] as f32);
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
            max_t: t,
            pos_interval: (pos1-0.1*pos_diff,pos2+0.1*pos_diff),
            instant: instant,
            diagram: diagram,
        }
    }
}

#[derive(Debug)]
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

    pub fn get_cached_instant<'a>(&'a self, vm :&ViewModel, idx :usize, time :f32) -> Option<&'a Instant> {
        self.data.vecmap_get(idx)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SignalAspect { Stop, Proceed }
#[derive(Debug, Copy, Clone)]
pub enum SectionStatus { Free, Reserved, Occupied }
#[derive(Debug, Copy, Clone)]
pub enum SwitchStatus { Left, Right, Unknown }

#[derive(Debug)]
pub struct Instant {
    pub time :f32,
    pub trains :Vec<TrainInstant>,
    pub infrastructure: InfrastructureState,
}

#[derive(Debug)]
pub struct InfrastructureState {
    //pub signals :HashMap<PtA, SignalAspect>,
    //pub sections :HashMap<ObjectId, SectionStatus>,
    pub sections :Vec<(ObjectId, SectionStatus, Vec<(PtC,PtC)>)>,
    pub switches :HashMap<Pt, SwitchStatus>,
    pub object_state :HashMap<PtA, Vec<ObjectState>>,
    // TODO sight lines
}

impl Instant {
    pub fn from(time :f32, history :&History, dgraph :&DGraph) -> Instant {
        Instant { time:time, trains: draw_train(time as f64,history,dgraph),
        infrastructure: draw_infrastructure(time as f64, history, dgraph)}
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
    pub end_kms :[f64;4],
    pub start_vel :f64,
    pub dt: f64,
    pub acc :f64,
}

pub fn get_km(dgraph :&DGraph, a :rolling_inf::NodeId, b :rolling_inf::NodeId, offset :f64) -> Option<f64> {
    let edge_length = edge_length(&dgraph.rolling_inf, a, b)?;
    let km1 = dgraph.mileage.get(&a)?;
    let km2 = dgraph.mileage.get(&b)?;
    let param = glm::clamp_scalar(offset / edge_length, 0.0, 1.0);
    Some(glm::lerp_scalar(*km1,*km2,param))
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

    // Any boxes that are still reserved or occupied should also be painted
    for (tvd, (reserved_t, (occupied_t, occ_node))) in occupied {
        if let Some(tvd_interval) = tvd_max_pos_interval(dgraph, tvd) {
            output.push(BlockGraph {
                pos: tvd_interval,
                reserved: (reserved_t, t),
                occupied: (occupied_t,t),
                train: 0,
                info: format!("info"), // TODO
            });
        }
    }
    for (tvd, (reserved_t, (occupied_t, occ_node), (vac_t, vac_node))) in vacant {
        if let Some(pos1) = dgraph.mileage.get(&occ_node) {
            if let Some(pos2) = dgraph.mileage.get(&vac_node) {
                output.push(BlockGraph {
                    pos: (pos1.min(*pos2), pos1.max(*pos2)),
                    reserved: (reserved_t, t),
                    occupied: (occupied_t,vac_t),
                    train: 0,
                    info: format!("info"), // TODO
                });
            }
        }
    }

    output
}

pub fn tvd_max_pos_interval(dgraph :&DGraph, tvd :rolling_inf::ObjectId) -> Option<(f64,f64)> {
    let (mut a, mut b) = (std::f64::INFINITY, -std::f64::INFINITY);
    for node_id in dgraph.tvd_entry_nodes.get(&tvd)?.iter() {
        if let Some(km) = dgraph.mileage.get(node_id) {
            a = a.min(*km);
            b = b.max(*km);
        }
    }
    Some((a,b))
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
        let mut edges_occupied = Vec::new();
        for e in events {
            match e {
                //TODO sight?
                TrainLogEvent::Wait(dt) => { t += dt; },
                TrainLogEvent::Edge(a,b) => {
                    edges_occupied.push(((*a,*b), 0.0, 0.0)); 
                    edge_x = 0.0;
                    if let Some(b) = b {
                        current_edge_pos = Some((*dgraph.mileage.get(a).unwrap(),
                                            *dgraph.mileage.get(b).unwrap(),
                                            edge_length(&dgraph.rolling_inf, *a, *b).unwrap()));
                    } else {
                        let km_dir = segments.last().map(|s:&TrainGraphSegment| 
                                (s.kms[3] - s.kms[0]).signum()).unwrap_or(1.);
                        current_edge_pos = Some((*dgraph.mileage.get(a).unwrap(),
                                            dgraph.mileage.get(a).unwrap() + km_dir*1000.0,
                                            1000.0));
                    }
                },
                TrainLogEvent::Move(dt,action,DistanceVelocity { dx, v }) => {
                    let acc = if *dt > 0.0 { (*v - prev_v)/dt } else { 0.0 };
                    if let Some((pos1,pos2,edge_length)) = current_edge_pos {
                        let mut kms = [0.;4];
                        let mut end_kms = [0.;4];
                        let (mut sample_x, mut sample_v) = (edge_x, prev_v);
                        for i in 0..=3 {
                            //println!("pos {}-{}-{}", pos1,pos2,edge_length);
                            //println!("i {}", i);
                            //println!("edge_list {:?}", edges_occupied);
                            kms[i] = glm::lerp_scalar(pos1, pos2, sample_x/edge_length);
                            let km_dir = (pos2 - pos1).signum();
                            let end_km = get_end_km(dgraph, params.length, &edges_occupied, km_dir);
                            if let Some(end_km) = end_km {
                                end_kms[i] = end_km;
                            } else {
                                println!("Warning: could not calculate train rear end position.");
                            }

                            //println!("front_km {:?}", kms[i]);
                            //println!("back_km {:?}", end_kms[i]);
                            //println!("---");

                            if i < 3 {


                            // TODO use dynamic_update from rolling instead of calculating
                            let dt = dt / 3.0;
                            let dx = sample_v * dt + 0.5 * acc * dt * dt;
                            sample_x += dx;
                            edges_occupied.last_mut().unwrap().2 += dx;
                            truncate_edge_list(&mut edges_occupied, params.length);
                            sample_v += acc * dt;
                            }
                        }

                        segments.push(TrainGraphSegment { 
                            start_time: t,
                            start_vel: prev_v,
                            dt: *dt,
                            kms: kms,
                            end_kms: end_kms,
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

pub fn draw_infrastructure(time :f64, history :&History, dgraph :&DGraph) -> InfrastructureState  {
    let mut object_state = HashMap::new();
    //let mut signals :HashMap<PtA, SignalAspect> = HashMap::new();

    for (obj_id, pta) in dgraph.object_ids.iter() {
        if let rolling_inf::StaticObject::Signal { has_distant } = dgraph.rolling_inf.objects[*obj_id] {
            let mut v = vec![ObjectState::SignalStop];
            if has_distant { v.push(ObjectState::DistantStop); }
            object_state.insert(*pta, v);
        }
    }

    let mut sections :HashMap<ObjectId, SectionStatus> = HashMap::new();
    let switches :HashMap<Pt, SwitchStatus> = HashMap::new();

    let mut reserved : HashSet<ObjectId> = HashSet::new();
    let mut occupied : HashSet<ObjectId> = HashSet::new();

    let mut t = 0.0;
    for infevent in &history.inf {
        use rolling::output::history::*;
        match infevent {
            InfrastructureLogEvent::Wait(dt) => { t += dt; if t > time { break; } },
            InfrastructureLogEvent::Authority(sig_d,(main,dist)) => {
                if let Some(pta) = dgraph.object_ids.get(sig_d) {
                    let state = vec![
                        if main.is_some() { ObjectState::SignalProceed } else { ObjectState::SignalStop },
                        if dist.is_some() { ObjectState::DistantProceed } else { ObjectState::DistantStop },
                    ];
                    object_state.insert(*pta,state);
                }
            },
            InfrastructureLogEvent::Reserved(tvd,b) => {
                if *b { reserved.insert(*tvd); } else { reserved.remove(tvd); }
            },
            InfrastructureLogEvent::Occupied(tvd,b,_,_) => {
                if *b { occupied.insert(*tvd); } else { occupied.remove(tvd); }
            },
            _ => {}, // TODO switches
        }
    }

    for tvd in reserved { sections.insert(tvd, SectionStatus::Reserved); }
    for tvd in occupied { sections.insert(tvd, SectionStatus::Occupied); }

    let mut sections_vec = Vec::new();
    for (tvd,status) in sections {
        let mut section_lines = Vec::new();
        if let Some(edges) = dgraph.tvd_edges.get(&tvd) {
            for edge in edges.iter() {
                if let Some(lines) = dgraph.edge_lines.get(edge) {
                    for (p1,p2) in lines.iter().zip(lines.iter().skip(1)) {
                        section_lines.push((*p1,*p2));
                    }
                }
            }
        }
        sections_vec.push((tvd,status,section_lines));
    }

    InfrastructureState { sections: sections_vec, switches, object_state }
}




#[derive(Debug)]
pub struct TrainInstant {
    pub lines :Vec<(PtC,PtC)>,
    pub signals_sighted: Vec<PtA>,
}

impl TrainInstant {
    pub fn get_front(&self) -> Option<PtC> {
        self.lines.last().map(|x| x.1)
    }
}

pub fn draw_train(time :f64, history :&History, dgraph :&DGraph) -> Vec<TrainInstant> {
    let mut trains = Vec::new();
    for (train_i, (name, params, events)) in history.trains.iter().enumerate() {

        use rolling::railway::dynamics::*;
        use rolling::output::history::*;
        let mut t = 0.0;
        let mut edges = Vec::new();
        let mut velocity = 0.0;

        let mut lines = Vec::new();
        let mut sighted :HashSet<PtA> = HashSet::new();
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
                TrainLogEvent::Sight(id, value) => {
                    let pta = dgraph.object_ids[id];
                    if *value { sighted.insert(pta); } else { sighted.remove(&pta); }
                },
                _ => {},
            }

            if t >= time { break; }
        }

        for e in edges {
            if let Some(line) = draw_edge_with_offset(dgraph, e) {
                lines.extend(line);
            }
        }

        trains.push(TrainInstant {
            lines: lines,
            signals_sighted: sighted.into_iter().collect(),
        });
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


fn get_end_km(dgraph :&DGraph, train_length: f64, edges :&[((usize,Option<usize>), f64, f64)], km_dir: f64) -> Option<f64> {
    let total_length = edges.iter().map(|(_,a,b)| (b-a).abs()).sum::<f64>();
    let add = -km_dir * (train_length - total_length);
    let ((a,b),offset,_) = edges.iter().cloned().next()?;
    let pos_a = dgraph.mileage.get(&a)?;
    if let Some(b) = b {
        let edge_length = edge_length(&dgraph.rolling_inf, a, b)?;
        let pos_b = dgraph.mileage.get(&b)?;
        Some(glm::lerp_scalar(*pos_a, *pos_b, glm::clamp_scalar(offset / edge_length, 0.0, 1.0)) + add)
    } else {
        Some(pos_a + km_dir * offset + add)
    }
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


