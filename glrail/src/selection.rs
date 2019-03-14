use crate::app::*;

#[derive(PartialEq, Debug)]
pub enum Selection {
    None,
    Entity(EntityId),
    Pos(Pos,f32,EntityId),
    PosRange(Pos,Pos),
    Path(()), // TODO
    Area(()), // TODO
}
