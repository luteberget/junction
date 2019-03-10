use crate::app::*;

#[derive(PartialEq)]
pub enum Selection {
    None,
    Object(EntityId),
    Pos(Pos,f32,EntityId),
    PosRange(Pos,Pos),
    Path(()), // TODO
    Area(()), // TODO
}
