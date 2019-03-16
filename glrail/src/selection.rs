use crate::app::*;
use crate::model::*;
use crate::infrastructure::*;

#[derive(PartialEq, Debug)]
pub enum Selection {
    None,
    Entity(EntityId),
    Pos(Pos,f32,EntityId),
    PosRange(Pos,Pos),
    Path(()), // TODO
    Area(()), // TODO
}
