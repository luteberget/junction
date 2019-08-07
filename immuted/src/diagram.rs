use const_cstr::*;
use crate::model::*;
use crate::viewmodel::*;
use crate::ui;
use std::sync::Arc;

use backend_glfw::imgui::*;

pub struct Diagram {
    history :Arc<History>,
    time_interval: (f32,f32),
    pos_interval: (f32,f32),
    time: f32,
    data :Vec<TrainGraph>,
}

impl Diagram {
    pub fn from_history(history :Arc<History>) -> Diagram {
        let t = max_time(&history) as f32;
        let x = max_pos(&history) as f32;
        let plot = plot_trains(&history);
        Diagram {
            history: history,
            time_interval: (0.0, t),
            pos_interval: (0.0,x),
            time: 0.0,
            data: plot,
        }
    }

    pub fn draw(&mut self, doc :&mut ViewModel) { unsafe {
        let format = const_cstr!("%.3f").as_ptr();
        igSliderFloat(const_cstr!("Time").as_ptr(), &mut self.time, 
                      self.time_interval.0, self.time_interval.1, format, 1.0);

        let size = igGetContentRegionAvail_nonUDT2().into();
        ui::canvas(size, const_cstr!("diagramcanvas").as_ptr(), |draw_list, pos| { 
            self.draw_background(&doc, draw_list, pos, size);
        });
    } }


    pub fn draw_background(&self, vm :&ViewModel, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2) {
        unsafe {
        let m = vm.get_undoable().get();
        let d = vm.get_data();
        let h = &self.history;
        for graph in &self.data {
            for s in &graph.segments {
                let p0 = (s.start_time, s.start_pos, s.start_vel);
                let dt = s.dt/3.0;
                let p1 = (p0.0 + dt, p0.1 + p0.2*dt + s.acc*dt*dt*0.5, p0.2 + s.acc*dt);
                let p2 = (p1.0 + dt, p1.1 + p1.2*dt + s.acc*dt*dt*0.5, p1.2 + s.acc*dt);
                let p3 = (p2.0 + dt, p2.1 + p2.2*dt + s.acc*dt*dt*0.5, p2.2 + s.acc*dt);
                draw_interpolate(draw_list,
                                 pos + self.to_screen(&size, p0.0, p0.1),
                                 pos + self.to_screen(&size, p1.0, p1.1),
                                 pos + self.to_screen(&size, p2.0, p2.1),
                                 pos + self.to_screen(&size, p3.0, p3.1));
            }
        }
        }
    }

    fn to_screen(&self, size :&ImVec2, t :f64, x :f64) -> ImVec2 {
        ImVec2 { x: size.x*(t as f32 - self.time_interval.0)/(self.time_interval.1 - self.time_interval.0),
                 y: size.y - size.y*(x as f32 - self.pos_interval.0)/(self.pos_interval.1 - self.pos_interval.0) }
    }
}

pub fn draw_interpolate(draw_list :*mut ImDrawList, p0 :ImVec2, y1 :ImVec2, y2 :ImVec2, p3 :ImVec2) {
    // https://web.archive.org/web/20131225210855/http://people.sc.fsu.edu/~jburkardt/html/bezier_interpolation.html
    let p1 = (-5.0*p0 + 18.0*y1 - 9.0*y2 + 2.0*p3) / 6.0;
    let p2 = (-5.0*p3 + 18.0*y2 - 9.0*y1 + 2.0*p0) / 6.0;
    unsafe {
    ImDrawList_AddBezierCurve(draw_list, p0,p1,p2,p3, ui::col::unselected(), 2.0, 0);
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


struct TrainGraph {
    segments :Vec<TrainGraphSegment>,
}


#[derive(Debug)]
struct TrainGraphSegment {
    start_time :f64,
    start_pos :f64,
    start_vel :f64,
    dt: f64,
    acc :f64,
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
                //TrainLogEvent::Edge(a,b) => { edges.push((a,b), 0.0, 0.0)); },
                //TrainLogEvent::Move(dt, action, DistanceVelocity { dx, v }) => {
                    //let update_x = if t + *dt < time { *dx } else {
                        //dynamic_update(params, velocity,DriverPlan { action: *action, dt: time - t}).dx
                    //};
                    //edges.last_mut().unwrap().2 += update_x;
                    //truncate_edge_list(&mut edges, params.length);
                    //velocity = *v;
                    //t += *dt;
                //},
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

                //TrainLogEvent::Sight(id,value) => {
                //},
                _ => {},
            }
        }
        output.push(TrainGraph { segments });
    }
    output
}







