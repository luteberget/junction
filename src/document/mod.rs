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
pub mod canvas;
pub mod view;
//pub mod diagram;


use crate::file;
use crate::app::*;

pub struct Document {
    pub viewmodel :viewmodel::ViewModel,
    pub fileinfo :file::FileInfo,
    //pub canvas :Canvas,
    pub dispatch :Option<Dispatch>,
}

impl Document {
    pub fn empty(bg :BackgroundJobs) -> Self {
        Document {
            viewmodel: viewmodel::ViewModel::from_model(model::Model::empty(), bg),
            fileinfo: file::FileInfo::empty(),
            dispatch: None,
        }
    }

    pub fn from_model(model :model::Model, bg: BackgroundJobs) -> Self {
        Document {
            viewmodel: viewmodel::ViewModel::from_model(model::Model::empty(), bg),
            fileinfo: file::FileInfo::empty(),
            dispatch: None,
        }
    }

    pub fn model(&self) -> &model::Model {
        self.viewmodel.get_undoable().get()
    }
}

pub enum Dispatch {
}

impl UpdateTime for Dispatch {
    fn advance(&mut self, dt :f64) {}
}

