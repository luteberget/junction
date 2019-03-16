use crate::selection::*;

pub enum InputDir {
    Up, Down, Left, Right
}

pub struct View {
    pub viewport : ((f64,f64),f64),
    pub selection :Selection,
    pub hot_route :Option<usize>,
    pub selected_movement :Option<usize>,
    pub selected_dispatch :Option<usize>,
    pub time :f32,
}

impl View {
    pub fn new_default() -> Self {
        View {
            viewport: ((0.5,2.0),10.0),
            selection: Selection::None,
            hot_route: None,
            selected_movement: None,
            selected_dispatch: None,
            time :0.0,
        }
    }
}

