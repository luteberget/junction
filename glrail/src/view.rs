use crate::selection::*;
use crate::infrastructure::*;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum InputDir {
    Up, Down, Left, Right
}

#[derive(Serialize, Deserialize)]
pub struct View {
    pub viewport : ((f64,f64),f64),
    pub selection :Selection,
    pub hot_route :Option<usize>,
    pub selected_dispatch :Option<usize>,
    pub selected_movement :Option<(usize,Option<usize>)>,
    pub canvas_context_menu_item :Option<EntityId>,
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
            canvas_context_menu_item: None,
            time :0.0,
        }
    }
}

