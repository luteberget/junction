use const_cstr::*;
use std::sync::mpsc;
use std::sync::Arc;

use crate::gui::widgets;
use crate::document::model::*;
use crate::document::analysis::*;
use crate::synthesis::*;
use crate::app::*;

pub struct SynthesisWindow {
    model :Arc<Model>,
    msgs :Vec<Result<FullSynMsg, ()>>,
    thread: Option<mpsc::Receiver<FullSynMsg>>,
    thread_pool: BackgroundJobs,
}

fn add_objects(analysis :&mut Analysis, objs :&Design) {
    for (track_idx, pos, func, dir) in objs.iter() {
    }
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

            full_synthesis(&SynthesisBackground { topo: &topo, plans: &plans, vehicles: &vehicles }, 
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
