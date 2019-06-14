
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash,PartialOrd,Ord)]
pub struct Pt {
    pub x: i32,
    pub y: i32,
}


pub type Vc = Pt;

pub fn pt_sub(to :Pt, from :Pt) -> Vc {
    Pt { x: to.x - from.x,
         y: to.y - from.y }
}

pub fn v_angle(v :Vc) -> i8 {
    match (v.x.signum(),v.y.signum()) {
        ( 1, 0) => 0,
        ( 1, 1) => 1,
        ( 0, 1) => 2,
        (-1, 1) => 3,
        (-1, 0) => 4,
        (-1,-1) => 5,
        ( 0,-1) => 6,
        ( 1,-1) => 7,
        _ => panic!()
    }
}

