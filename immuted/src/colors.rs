use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
pub enum RailUIColor {
    TVDFree,
    TVDOccupied,
    TVDReserved,
}


pub fn colr_no(x :usize) -> RailUIColor {
    x.into().unwrap()
}

