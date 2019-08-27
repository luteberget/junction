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

