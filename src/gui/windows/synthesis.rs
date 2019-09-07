use nalgebra_glm as glm;
use ordered_float::OrderedFloat;
use const_cstr::*;
use std::sync::mpsc;
use std::sync::Arc;

use crate::gui::widgets;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::document::objects;
use crate::document::infview::round_coord;
use crate::synthesis::*;
use crate::app::*;

pub struct SynthesisWindow {
    model :Arc<Model>,
    msgs :Vec<Result<FullSynMsg, ()>>,
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
        println!("ADding object {:?}", obj);
        model.objects.insert(round_coord(obj.loc), obj);
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
        SynthesisWindow {
            model: Arc::new(model),
            msgs: Vec::new(),
            thread: None,
            thread_pool: bg,
        }
    }

    pub fn draw(&mut self, current_doc :&mut Analysis) -> bool {
        let mut keep_open = true;
        use backend_glfw::imgui::*;
        unsafe {
            igBegin(const_cstr!("Signal designer").as_ptr(), &mut keep_open as _, 0 as _);
            widgets::show_text("Got model.");

            if self.thread.is_none() {
                if igButton(const_cstr!("Start synthesis.").as_ptr(), ImVec2 { x: 120.0, y: 0.0 }) {
                    self.start();
                }
            } else {
                widgets::show_text("Running.");
            }

            for (msg_i,msg) in self.msgs.iter().enumerate() {
                igPushIDInt(msg_i as _);

                match msg {
                    Ok(FullSynMsg::ModelAvailable(n,score,objs)) => {
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

                        igSameLine(0.0,-1.0); widgets::show_text(&format!("Model {} @ {} with {} objs.",
                                                                          n, score, objs.len()));
                    },
                    Ok(msg) => { widgets::show_text(&format!("{:?}",msg)); }
                    Err(()) => {
                        widgets::show_text("Process finished.");
                    },
                }

                igPopID();
            }

            igEnd();
        }

        keep_open
    }

    pub fn start(&mut self) {
        self.msgs = Vec::new();
        let (tx,rx) = mpsc::channel();
        self.thread = Some(rx);
        let model = self.model.clone();
        self.thread_pool.execute(move || {
            use crate::document::topology;
            let topo = topology::convert(&model, 50.0).unwrap();
            let plans = model.plans.iter().map(|(_id,p)| p).cloned().collect::<Vec<_>>();
            let vehicles = model.vehicles.iter().cloned().collect::<Vec<_>>();

            full_synthesis(&SynthesisBackground { topology: &topo, plans: &plans, vehicles: &vehicles }, 
                           |msg| tx.send(msg).is_ok()).unwrap(); // TODO unwrap?
        });
    }
}


impl BackgroundUpdates for SynthesisWindow {
    fn check(&mut self) {
        if let Some(rx) = &mut self.thread {
            loop {
                match rx.try_recv() {
                    Ok(msg) => { self.msgs.push(Ok(msg)); },
                    Err(mpsc::TryRecvError::Disconnected) => { 
                        self.thread = None;
                        self.msgs.push(Err(())); 
                        break;
                    },
                    Err(mpsc::TryRecvError::Empty) => { break; }
                }
            }
        }
    }
}
