use log::*;
use std::sync::mpsc::*;
use std::collections::HashMap;

use crate::document::model::*;
use crate::document::dgraph::*;
use crate::document::topology;
use crate::document::interlocking;
use crate::document::infview::unround_coord;

use crate::document::history;
use crate::app;
use crate::util;
use crate::util::VecMap;
use crate::document::dispatch;
use crate::document::plan;
use std::sync::Arc;
use nalgebra_glm as glm;

pub type Generation = usize;

#[derive(Default)]
pub struct AnalysisOutput {
    pub topology: Option<(Generation, Arc<topology::Topology>)>,
    pub dgraph :Option<(Generation, Arc<DGraph>)>,
    pub interlocking :Option<(Generation, Arc<interlocking::Interlocking>)>,
    pub dispatch :Vec<Option<(Generation, dispatch::DispatchOutput)>>,
    pub plandispatches :HashMap<usize, Vec<Option<(Generation, dispatch::DispatchOutput)>>>,
}

pub struct Analysis {
    pub model: Undoable<Model, EditClass>,
    model_generation: Generation,
    pub output: AnalysisOutput,
    chan :Option<Receiver<SetData>>,
    bg :app::BackgroundJobs,
}

#[derive(Debug)]
pub enum SetData {
    DGraph(Generation, Arc<DGraph>),
    Interlocking(Generation, Arc<interlocking::Interlocking>),
    Dispatch(Generation, usize,dispatch::DispatchOutput),
    PlanDispatch(Generation, usize,usize,dispatch::DispatchOutput),
}

impl app::BackgroundUpdates for Analysis {
    fn check(&mut self) {
        while let Some(Ok(data)) = self.chan.as_mut().map(|r| r.try_recv()) {
            match data {
                SetData::DGraph(g, dgraph) => { self.output.dgraph = Some((g, dgraph)); },
                SetData::Interlocking(g, il) => { self.output.interlocking = Some((g, il)); },
                SetData::Dispatch(g, idx,h) => { 
                    self.output.dispatch.vecmap_insert(idx, (g, h));
                    //cache.clear_dispatch(idx);
                },
                SetData::PlanDispatch(g, plan_idx,dispatch_idx,h) => {
                    self.output.plandispatches.entry(plan_idx)
                        .or_insert(Vec::new())
                        .vecmap_insert(dispatch_idx, (g, h));
                },
            }
        }
    }
}

impl Analysis {
    pub fn model(&self) -> &Model { &self.model.get() }
    pub fn data(&self) -> &AnalysisOutput { &self.output }

    pub fn from_model(model :Model, bg: app::BackgroundJobs) -> Self {
        Analysis {
            model: Undoable::from(model),
            model_generation: 0,
            output: Default::default(),
            chan: None,
            bg: bg,
        }
    }

    fn update(&mut self) {
        let model = self.model.get().clone(); // persistent structs
        let gen = self.model_generation;

        let topology = Arc::new(topology::convert(&model, 50.0).unwrap());
        self.output.topology = Some((gen,topology.clone()));

        let (tx,rx) = channel();
        self.chan = Some(rx);

        self.bg.execute(move || {
            info!("Background thread starting");
            let model = model;  // move model into thread
            let tx = tx;        // move sender into thread

            //let dgraph = dgraph::calc(&model); // calc dgraph from model.
            let dgraph = DGraphBuilder::convert(&topology).expect("dgraph conversion failed");
            let dgraph = Arc::new(dgraph);

            info!("Dgraph successful with {:?} nodes", dgraph.rolling_inf.nodes.len());

            let send_ok = tx.send(SetData::DGraph(gen, dgraph.clone()));
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
            let send_ok = tx.send(SetData::Interlocking(gen, interlocking.clone()));
            if !send_ok.is_ok() { println!("job canceled after interlocking"); return; }
            info!("Interlocking successful with {:?} routes", interlocking.routes.len());

            for (i,dispatch) in model.dispatches.iter() {
                //let history = dispatch::run(&dgraph, &interlocking, &dispatch);
                let (history,route_refs) = history::get_history(model.vehicles.data(),
                                                   &dgraph.rolling_inf,
                                                   &interlocking,
                                                   &(dispatch.commands)).unwrap();
                info!("Simulation successful {:?}", &dispatch.commands);
                let view = dispatch::DispatchOutput::from_history(dispatch.clone(), &dgraph, history);
                let send_ok = tx.send(SetData::Dispatch(gen, *i, view));
                if !send_ok.is_ok() { println!("job canceled after dispatch"); return; }
            }

            for (plan_idx,plan) in model.plans.iter() {
                let dispatches = plan::get_dispatches(&interlocking,
                                             model.vehicles.data(),
                                             plan).unwrap();

                info!("Planning successful. {:?}", dispatches);

                for (dispatch_idx,d) in dispatches.into_iter().enumerate() {
                    let (history, route_refs) = history::get_history(model.vehicles.data(),
                                         &dgraph.rolling_inf,
                                         &interlocking,
                                         &d.commands).unwrap(); // TODO UNWRAP?
                    info!("Planned simulation successful");
                    let view = dispatch::DispatchOutput::from_history(d.clone(), &dgraph, history);
                    let send_ok = tx.send(SetData::PlanDispatch(gen, *plan_idx, dispatch_idx, view));
                    if !send_ok.is_ok() { println!("job cancelled after plan dispatch {}/{}", 
                                                   plan_idx, dispatch_idx); }
                }

            }

        });
    }

    pub fn edit_model(&mut self, mut f :impl FnMut(&mut Model) -> Option<EditClass>) {
        let mut new_model = self.model.get().clone();
        let cl = f(&mut new_model);
        self.set_model(new_model, cl);
    }

    pub fn set_model(&mut self, m :Model, cl :Option<EditClass>) {
        info!("Updating model");
        self.model.set(m, cl);
        self.on_changed();
    }

    pub fn override_edit_class(&mut self, cl :EditClass) {
        self.model.override_edit_class(cl);
    }

    pub fn undo(&mut self) { if self.model.undo() { self.on_changed(); } }
    pub fn redo(&mut self) { if self.model.redo() { self.on_changed(); } }

    fn on_changed(&mut self) {
        // TODO 
        // kself.fileinfo.set_unsaved();
        self.model_generation += 1;
        self.update();
    }


    pub fn get_rect(&self, a :PtC, b :PtC) -> Vec<Ref> {
        let mut r = Vec::new();
        for (a,b) in self.model().get_linesegs_in_rect(a,b) {
            r.push(Ref::LineSeg(a,b));
        }
        if let Some((_,topo)) = self.data().topology.as_ref() {
            for (pt,_) in topo.locations.iter() {
                if util::in_rect(glm::vec2(pt.x as f32,pt.y as f32), a,b) {
                    r.push(Ref::Node(*pt));
                }
            }
        }
        for (pta,_) in self.model().objects.iter() {
            if util::in_rect(unround_coord(*pta), a, b) {
                r.push(Ref::Object(*pta));
            }
        }
        r
    }

    pub fn get_closest(&self, pt :PtC) -> Option<(Ref,f32)> {
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        if let Some(((p1,p2),_param,(d,_n))) = self.model().get_closest_lineseg(pt) {
            thing = Some(Ref::LineSeg(p1,p2));
            dist_sqr = d;
        }

        if let Some((p,d)) = self.get_closest_node(pt) {
            if d < 0.5*0.5 {
                thing = Some(Ref::Node(p));
                dist_sqr = d;
            }
        }

        if let Some(((p,_obj),d)) = self.model().get_closest_object(pt) {
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
            for (px,_) in self.data().topology.as_ref()?.1.locations.iter() {
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

