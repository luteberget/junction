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
    pub selected_scenario :SelectedScenario,
    pub canvas_context_menu_item :Option<EntityId>,
    pub time :f32,
}

#[derive(PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum SelectedScenario {
    None,
    Dispatch(usize),
    Usage(usize, Option<usize>),
}

impl SelectedScenario {
    pub fn has_dispatch(&self) -> bool  {
        match self {
            SelectedScenario::Dispatch(_) => true,
            SelectedScenario::Usage(_, Some(_)) => true,
            _ => false,
        }
    }
}

impl View {
    pub fn new_default() -> Self {
        View {
            viewport: ((0.5,2.0),10.0),
            selection: Selection::None,
            hot_route: None,
            selected_scenario: SelectedScenario::None,
            canvas_context_menu_item: None,
            time :0.0,
        }
    }
}

