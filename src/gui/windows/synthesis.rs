use matches::matches;
use nalgebra_glm as glm;
use ordered_float::OrderedFloat;
use const_cstr::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use log::*;

use crate::gui::widgets;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::document::objects;
use crate::document::objects::*;
use crate::document::infview::round_coord;
use crate::synthesis::*;
use crate::app::*;

pub struct SynthesisWindow {
    model :Arc<Model>,

    result_models :Vec<(String, f64, Design)>,
    results_ranking :Vec<usize>,
    results_log :Vec<String>,

    enabled_planspecs :HashMap<usize,bool>,

    thread: Option<mpsc::Receiver<FullSynMsg>>,
    thread_pool: BackgroundJobs,
}

fn add_objects(analysis :&mut Analysis, objs :&Design) {
    use crate::document::topology;
    let mut model = analysis.model().clone();
    let topo = topology::convert(&model, 50.0).unwrap();
    for (track_idx, pos, func, dir) in objs.iter() {
        let sideways = match dir {
            None => 0.0,
            Some(AB::A) => 0.01,
            Some(AB::B) => -0.01,
        };

        let (pt,tangent) = loc_on_track(&topo.interval_lines, *track_idx, *pos);
        let normal = glm::vec2(tangent.y, -tangent.x);
        let mut obj = objects::Object {
            loc: pt, 
            tangent: glm::vec2(tangent.x.round() as _, tangent.y.round() as _),
            functions: vec![*func],
        };
        obj.move_to(&model, pt + sideways*glm::vec2(normal.x as f32, normal.y as f32));
        //println!("ADding object {:?}", obj);
        model.objects.insert(round_coord(obj.loc), obj);

        if matches!(func, Function::MainSignal { .. } ) {
            let mut obj = objects::Object {
                loc: pt, 
                tangent: glm::vec2(tangent.x.round() as _, tangent.y.round() as _),
                functions: vec![Function::Detector],
            };
            obj.move_to(&model, pt + sideways*glm::vec2(normal.x as f32, normal.y as f32));
            //println!("ADding object {:?}", obj);
            model.objects.insert(round_coord(obj.loc), obj);
        }
    }

    analysis.set_model(model, None);
}

fn loc_on_track(interval_lines :&Vec<Vec<(OrderedFloat<f64>, PtC)>>, track_idx :usize, l :f64) -> (PtC, PtC) {
    let lines = &interval_lines[track_idx];
    for ((OrderedFloat(l_a),p_a),(OrderedFloat(l_b),p_b)) in lines.iter().zip(lines.iter().skip(1)) {
        if *l_a <= l && l <= *l_b {
            let pt = glm::lerp(p_a,p_b,((l - l_a)/(l_b - l_a)) as f32);
            let tangent = p_b - p_a;
            return (pt,tangent);
        }
    }
    panic!()
}

impl SynthesisWindow {
    pub fn new(model :Model, bg :BackgroundJobs) -> SynthesisWindow {
        let mut win = SynthesisWindow {
            model: Arc::new(model),
            result_models :Vec::new(),
            results_ranking :Vec::new(),
            results_log :Vec::new(),

            enabled_planspecs :HashMap::new(),
            thread: None,
            thread_pool: bg,
        };
        win.start();
        win
    }

    pub fn draw(&mut self, current_doc :&mut Analysis) -> bool {
        let mut keep_open = true;
        use backend_glfw::imgui::*;
        unsafe {
            widgets::next_window_center_when_appearing();
            igBegin(const_cstr!("Signal designer").as_ptr(), &mut keep_open as _, 0 as _);
            let mut window_size = igGetContentRegionAvail_nonUDT2();
            igBeginChild(const_cstr!("sdl").as_ptr(), 
                         ImVec2 { x: window_size.x/2.0, y: -150.0 }, true, 0 as _);

            widgets::show_text("\u{f0d0} Use plans:");

            for (plan_id,plan) in current_doc.model().plans.iter() {
                igPushIDInt(*plan_id as _);
                let mut active = self.enabled_planspecs.get(plan_id).cloned().unwrap_or(true);
                igCheckbox(const_cstr!("").as_ptr(), &mut active as _);
                if igIsItemEdited() {
                    self.enabled_planspecs.insert(*plan_id, active);
                    self.start();
                }
                igSameLine(0.0,-1.0);
                widgets::show_text(&plan.name);
                igPopID();
            }

            igEndChild();
            igSameLine(0.0,-1.0);
            igBeginChild(const_cstr!("sdr").as_ptr(), ImVec2 { x: 0.0, y: -150.0 }, true, 0 as _);
            if self.thread.is_some() {
                widgets::show_text("\u{f110} Running.");
            } else {
                if self.result_models.len() > 0 {
                    widgets::show_text("\u{f00c} Designs available.");
                } else {
                    widgets::show_text("\u{f00d} No solutions found.");
                }
            }

            for i in self.results_ranking.iter() {
                igPushIDInt(*i as _);
                let (n,score,objs) = &self.result_models[*i];
                if igSelectable(const_cstr!("##msg").as_ptr(), false, 0 as _, ImVec2::zero()) {
                    add_objects(current_doc, objs);
                }

                if igIsItemHovered(0) {
                    igBeginTooltip();
                    igPushTextWrapPos(300.0);
                    widgets::show_text(&format!("{:?}", objs));
                    igPopTextWrapPos();
                    igEndTooltip();
                }

                igSameLine(0.0,-1.0); widgets::show_text(&format!("Design {} @ {:.2} with {} objs.",
                                                                  n, score, objs.len()));
                igPopID();
            }

            igEndChild();
            igBeginChild(const_cstr!("sdlog").as_ptr(), ImVec2::zero(), true, 0 as _);
            for (i,msg) in self.results_log.iter().enumerate().rev() {
                igPushIDInt(i as _);
                widgets::show_text(msg);
                igPopID();
            }
            igEndChild();

            igEnd();
        }

        keep_open
    }

    pub fn start(&mut self) {
        self.result_models = Vec::new();
        self.results_ranking = Vec::new();
        let (tx,rx) = mpsc::channel();
        self.thread = Some(rx);
        let model = self.model.clone();

        let plans = model.plans.iter()
            .filter_map(|(id,p)| if self.enabled_planspecs.get(id).cloned().unwrap_or(true) {
                Some(p) } else { None })
            .cloned().collect::<Vec<_>>();

        self.thread_pool.execute(move || {
            use crate::document::topology;
            let topo = topology::convert(&model, 50.0).unwrap();
            let vehicles = model.vehicles.iter().cloned().collect::<Vec<_>>();

            let result = full_synthesis(&SynthesisBackground { topology: &topo, plans: &plans, vehicles: &vehicles }, 
                           |msg| tx.send(msg).is_ok());

            if let Err(e) = result {
                error!("full_synthesis: {:?}", e);
            }

        });
    }
}


impl BackgroundUpdates for SynthesisWindow {
    fn check(&mut self) {
        if let Some(rx) = &mut self.thread {
            loop {
                match rx.try_recv() {
                    Ok(FullSynMsg::S(s)) => { 
                        self.results_log.push(s); 
                    },
                    Ok(FullSynMsg::ModelAvailable(a,b,c)) => { 
                        self.result_models.push((a,b,c)); 
                        self.results_ranking = (0..(self.result_models.len())).collect();
                        let m = &mut self.result_models;
                        self.results_ranking.sort_by_key(|i| OrderedFloat(m[*i].1));
                    }
                    Ok(_) => {},
                    Err(mpsc::TryRecvError::Disconnected) => { 
                        self.thread = None;
                        self.results_log.push(format!("Synthesis procedure finished."));
                        break;
                    },
                    Err(mpsc::TryRecvError::Empty) => { break; }
                }
            }
        }
    }
}
