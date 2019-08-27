use log::*;
use std::sync::mpsc::*;

use crate::document::model::*;
use crate::document::dgraph::*;
use crate::document::topology;
use crate::document::interlocking;
use crate::document::infview::unround_coord;

use crate::document::history;
use crate::util;
use crate::app;
use crate::util::VecMap;
use crate::document::dispatch;
use std::sync::Arc;
use nalgebra_glm as glm;

#[derive(Default)]
pub struct Derived {
    pub dgraph :Option<Arc<DGraph>>,
    pub interlocking :Option<Arc<interlocking::Interlocking>>,
    pub dispatch :Vec<Option<dispatch::DispatchView>>,
    pub topology: Option<Arc<topology::Topology>>,
}

pub struct ViewModel {
    pub(super) model: Undoable<Model, EditClass>,
    pub(super) derived :Derived,
    get_data :Option<Receiver<SetData>>,
    bg :app::BackgroundJobs,
}

#[derive(Debug)]
pub enum SetData {
    DGraph(Arc<DGraph>),
    Interlocking(Arc<interlocking::Interlocking>),
    Dispatch(usize,dispatch::DispatchView),
}

impl app::BackgroundUpdates for ViewModel {
    fn check(&mut self) {
        while let Some(Ok(data)) = self.get_data.as_mut().map(|r| r.try_recv()) {
            match data {
                SetData::DGraph(dgraph) => { self.derived.dgraph = Some(dgraph); },
                SetData::Interlocking(il) => { self.derived.interlocking = Some(il); },
                SetData::Dispatch(idx,h) => { 
                    self.derived.dispatch.vecmap_insert(idx,h);
                    //cache.clear_dispatch(idx);
                },
            }
        }
    }
}

impl ViewModel {
    pub fn from_model(model :Model, bg: app::BackgroundJobs) -> Self {
        ViewModel {
            model: Undoable::from(model),
            derived: Default::default(),
            get_data: None,
            bg: bg,
        }
    }

    pub(super) fn update(&mut self) {
        let model = self.model.get().clone(); // persistent structs
        let topology = Arc::new(topology::convert(&model, 50.0).unwrap());
        self.derived.topology = Some(topology.clone());

        let (tx,rx) = channel();
        self.get_data = Some(rx);
        self.bg.execute(move || {
            info!("Background thread starting");
            let model = model;  // move model into thread
            let tx = tx;        // move sender into thread

            //let dgraph = dgraph::calc(&model); // calc dgraph from model.
            let dgraph = DGraphBuilder::convert(&model,&topology).expect("dgraph conversion failed");
            let dgraph = Arc::new(dgraph);

            info!("Dgraph successful with {:?} nodes", dgraph.rolling_inf.nodes.len());

            let send_ok = tx.send(SetData::DGraph(dgraph.clone()));
            if !send_ok.is_ok() { println!("job canceled after dgraph"); return; }
            // if tx fails (channel is closed), we don't need 
            // to proceed to next step. Also, there is no harm
            // in *trying* to send the data from an obsolete thread,
            // because the update function will have replaced its 
            // receiver end of the channel, so it will anyway not
            // be placed into the struct.

            let interlocking = interlocking::calc(&dgraph); 
            let interlocking = Arc::new(interlocking);
                // calc interlocking from dgraph
            let send_ok = tx.send(SetData::Interlocking(interlocking.clone()));
            if !send_ok.is_ok() { println!("job canceled after interlocking"); return; }
            info!("Interlocking successful with {:?} routes", interlocking.routes.len());

            for (i,dispatch) in model.dispatches.iter().enumerate() {
                //let history = dispatch::run(&dgraph, &interlocking, &dispatch);
                let history = history::get_history(&model.vehicles,
                                                   &dgraph.rolling_inf,
                                                   interlocking.routes.iter().map(|(r,_)| r),
                                                   &(dispatch.0)).unwrap();
                info!("Simulation successful {:?}", &dispatch.0);
                let view = dispatch::DispatchView::from_history(&dgraph, history);
                let send_ok = tx.send(SetData::Dispatch(i, view));
                if !send_ok.is_ok() { println!("job canceled after dispatch"); return; }
            }
        });
    }


    pub fn get_rect(&self, a :PtC, b :PtC) -> Vec<Ref> {
        let mut r = Vec::new();
        for (a,b) in self.model.get().get_linesegs_in_rect(a,b) {
            r.push(Ref::LineSeg(a,b));
        }
        if let Some(topo) = self.derived.topology.as_ref() {
            for (pt,_) in topo.locations.iter() {
                if util::in_rect(glm::vec2(pt.x as f32,pt.y as f32), a,b) {
                    r.push(Ref::Node(*pt));
                }
            }
        }
        for (pta,_) in self.model.get().objects.iter() {
            if util::in_rect(unround_coord(*pta), a, b) {
                r.push(Ref::Object(*pta));
            }
        }
        r
    }

    pub fn get_closest(&self, pt :PtC) -> Option<(Ref,f32)> {
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        if let Some(((p1,p2),_param,(d,_n))) = self.model.get().get_closest_lineseg(pt) {
            thing = Some(Ref::LineSeg(p1,p2));
            dist_sqr = d; 
        }

        if let Some((p,d)) = self.get_closest_node(pt) {
            if d < 0.5*0.5 {
                thing = Some(Ref::Node(p));
                dist_sqr = d;
            }
        }

        if let Some(((p,_obj),d)) = self.model.get().get_closest_object(pt) {
            if d < 0.5*0.5 {
                thing = Some(Ref::Object(*p));
                dist_sqr = d;
            }
        }

        thing.map(|t| (t,dist_sqr))
    }

    pub fn get_closest_node(&self, pt :PtC) -> Option<(Pt,f32)> {
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        for p in corners(pt) {
            for (px,_) in self.derived.topology.as_ref()?.locations.iter() {
                if &p == px {
                    let d = glm::length2(&(pt-glm::vec2(p.x as f32,p.y as f32)));
                    if d < dist_sqr {
                        thing = Some(p);
                        dist_sqr = d;
                    }
                }
            }
        }
        thing.map(|t| (t,dist_sqr))
    }
}

