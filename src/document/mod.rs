// core model
pub mod model;
pub mod objects;

// derived data updates
pub mod viewmodel;

// derived data computation
pub mod dgraph;
pub mod topology;
pub mod interlocking;
pub mod history;
pub mod dispatch;
pub mod mileage;

// graphical view representation
pub mod infview;
pub mod view;
//pub mod diagram;

use crate::file;
use crate::app::*;
use model::*;
use infview::*;
use log::*;
use crate::util;
use crate::util::VecMap;
use nalgebra_glm as glm;

pub struct Document {
    viewmodel :viewmodel::ViewModel,
    pub fileinfo :file::FileInfo,
    pub inf_view :InfView,
    pub dispatch_view :Option<DispatchView>,
}

impl BackgroundUpdates for Document {
    fn check(&mut self) {
        self.viewmodel.check();
    }
}

impl Document {
    pub fn empty(bg :BackgroundJobs) -> Self {
        Self::from_model(model::Model::empty(), bg)
    }

    pub fn from_model(model :model::Model, bg: BackgroundJobs) -> Self {
        Document {
            viewmodel: viewmodel::ViewModel::from_model(model, bg),
            fileinfo: file::FileInfo::empty(),
            inf_view: InfView::default(),
            dispatch_view: None,
        }
    }

    pub fn model(&self) -> &model::Model {
        self.viewmodel.model.get()
    }

    pub fn data(&self) -> &viewmodel::Derived {
        &self.viewmodel.derived
    }

    pub fn edit_model(&mut self, mut f :impl FnMut(&mut Model) -> Option<EditClass>) {
        let mut new_model = self.viewmodel.model.get().clone();
        let cl = f(&mut new_model);
        self.set_model(new_model, cl);
    }

    pub fn set_model(&mut self, m :Model, cl :Option<EditClass>) {
        info!("Updating model");
        self.viewmodel.model.set(m, cl);
        self.on_changed();
    }

    pub fn override_edit_class(&mut self, cl :EditClass) {
        self.viewmodel.model.override_edit_class(cl);
    }

    pub fn undo(&mut self) { if self.viewmodel.model.undo() { self.on_changed(); } }
    pub fn redo(&mut self) { if self.viewmodel.model.redo() { self.on_changed(); } }

    fn on_changed(&mut self) {
        self.fileinfo.set_unsaved();
        self.viewmodel.update();
    }


    pub fn get_rect(&self, a :PtC, b :PtC) -> Vec<Ref> {
        let mut r = Vec::new();
        for (a,b) in self.model().get_linesegs_in_rect(a,b) {
            r.push(Ref::LineSeg(a,b));
        }
        if let Some(topo) = self.data().topology.as_ref() {
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
            for (px,_) in self.data().topology.as_ref()?.locations.iter() {
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

pub enum DispatchView {
    Manual(ManualDispatchView),
    Auto(AutoDispatchView),
}

pub struct ManualDispatchView {
}

pub struct AutoDispatchView {
}

impl UpdateTime for DispatchView {
    fn advance(&mut self, dt :f64) {}
}

