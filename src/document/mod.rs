pub mod model;
pub mod viewmodel;
pub mod dgraph;
pub mod topology;
pub mod interlocking;
pub mod canvas;
pub mod history;
pub mod dispatch;
pub mod objects;
pub mod mileage;
pub mod view;

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
}

pub enum Dispatch {
}

impl UpdateTime for Dispatch {
    fn advance(&mut self, dt :f64) {}
}

