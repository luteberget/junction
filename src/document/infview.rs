use std::collections::HashSet;
use crate::document::model::{Ref, PtA, PtC, Pt};
use nalgebra_glm as glm;
use crate::document::view::*;
use crate::document::objects::*;
use crate::gui::ImVec2;

#[derive(Debug)]
pub struct InfView {
    pub action :Action,
    pub selection :HashSet<Ref>,
    pub view :View,
    //pub instant_cache: dispatch::InstantCache,
}

#[derive(Debug)]
pub enum Action {
    Normal(NormalState),
    DrawingLine(Option<Pt>),
    InsertObject(Option<Object>),
}


#[derive(Debug,Copy,Clone)]
pub enum NormalState {
    Default,
    SelectWindow(ImVec2),
    DragMove(MoveType),
}


#[derive(Debug,Copy,Clone)]
pub enum MoveType { Grid(PtC), Continuous }

impl InfView {
    pub fn default() -> Self {
        InfView {
            action: Action::Normal(NormalState::Default),
            selection: HashSet::new(),
            view: View::default(),
        }
    }
}



pub fn unround_coord(p :PtA) -> PtC {
    let coeff = 10.0;
    glm::vec2(p.x as f32 / coeff, p.y as f32 / coeff)
}
pub fn round_coord(p :PtC) -> PtA {
    let coeff = 10.0;
    glm::vec2((p.x * coeff) as _, (p.y * coeff) as _)
}

