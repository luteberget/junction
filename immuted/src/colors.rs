use num_derive::FromPrimitive;

#[derive(FromPrimitive, Debug, PartialEq, Eq, Copy, Clone)]
pub enum RailUIColor {
    TVDFree,
    TVDOccupied,
    TVDReserved,
}


#[test]
pub fn colr_no() {
    use num_traits::FromPrimitive;
    let x = RailUIColor::from_usize(2);
    dbg!(x.unwrap());
    assert_eq!(x.unwrap(), RailUIColor::TVDReserved);
}



