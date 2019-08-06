use crate::model::*;
use crate::viewmodel::*;
use crate::ui;
use std::sync::Arc;

pub struct Diagram {
    history :Arc<History>,
    time_interval: (f64,f64),
    pos_interval: (f64,f64),
    data :(),
}

impl Diagram {
    pub fn from_history(history :Arc<History>) -> Diagram {
        unimplemented!()
    }

    pub fn draw(viewmodel :&mut ViewModel) {
    }
}

