use crate::app::*;
use crate::model::*;
use crate::infrastructure::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum Selection {
    None,
    Entity(EntityId),
    Pos(Pos,f32,EntityId),
    PosRange(Pos,Pos),
    Path(()), // TODO
    Area(()), // TODO
}
