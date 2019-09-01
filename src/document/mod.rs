// core model
pub mod model;
pub mod objects;

// derived data updates
pub mod analysis;

// derived data computation
pub mod dgraph;
pub mod topology;
pub mod interlocking;
pub mod history;
pub mod dispatch;
pub mod mileage;
pub mod plan;

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
use backend_glfw::imgui::ImVec2;

pub struct Document {
    pub analysis: analysis::Analysis,
    pub fileinfo :file::FileInfo,
    pub inf_view :InfView,
    pub dispatch_view :Option<DispatchView>,
}

impl BackgroundUpdates for Document {
    fn check(&mut self) {
        self.analysis.check();
    }
}

impl Document {
    pub fn empty(bg :BackgroundJobs) -> Self {
        Self::from_model(model::Model::empty(), bg)
    }

    pub fn from_model(model :model::Model, bg: BackgroundJobs) -> Self {
        Document {
            analysis: analysis::Analysis::from_model(model, bg),
            fileinfo: file::FileInfo::empty(),
            inf_view: InfView::default(),
            dispatch_view: None,
        }
    }


}

#[derive(Clone,Copy)]
pub enum DispatchView {
    Manual(ManualDispatchView),
    Auto(AutoDispatchView),
}

#[derive(Clone,Copy)]
pub struct ManualDispatchView {
    pub dispatch_idx :usize,
    pub time :f64,
    pub play :bool,
    pub viewport :Option<DiagramViewport>,
}

impl ManualDispatchView {
    pub fn new(idx :usize) -> ManualDispatchView {
        ManualDispatchView {
            dispatch_idx: idx,
            time: 0.0,
            play: false,
            viewport: None,
        }
    }
}

#[derive(Clone,Copy)]
pub struct DiagramViewport {
    pub time :(f64,f64),
    pub pos :(f64,f64),
}

#[derive(Clone,Copy)]
pub struct AutoDispatchView {
    pub plan_idx :usize,
    pub action :PlanViewAction,
    pub dispatch :Option<ManualDispatchView>,
}


#[derive(Debug, Copy, Clone)]
#[derive(PartialEq, Eq, Hash)]
pub struct VisitKey { 
    pub train: usize, 
    pub visit: usize, 
    pub location: Option<usize> 
}

#[derive(Clone,Copy)]
pub enum PlanViewAction {
    None,
    DragFrom(VisitKey, ImVec2),
}

impl UpdateTime for DispatchView {
    fn advance(&mut self, dt :f64) {}
}

