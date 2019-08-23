use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};
use serde::de::{Deserialize, Deserializer, Visitor};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash,PartialOrd,Ord)]
pub struct Pt {
    pub x: i32,
    pub y: i32,
}

impl Serialize for Pt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
        where S : Serializer
    {
        serializer.serialize_str(&format!("{};{}", self.x,self.y))
    }
}

impl<'de> Deserialize<'de> for Pt {
    fn deserialize<D>(deserializer: D) -> Result<Pt, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_str(PtDeser)
    }
}

struct PtDeser;

impl<'de> Visitor<'de> for PtDeser {
    type Value = Pt;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expecting a point x;y, e.g. \"4;-5\".")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        let components = s.split(";").collect::<Vec<_>>();
        match &components[..] {
            [x,y] => Ok(Pt { x: x.parse().map_err(|_| E::custom("int parse error"))?,
                             y: y.parse().map_err(|_| E::custom("int parse error"))? }),
            _ => panic!(),
        }
    }
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

pub fn angle_v(a :i8) -> Vc {
    match a {
        0 => Pt { x:  1, y:  0 },
        1 => Pt { x:  1, y:  1 },
        2 => Pt { x:  0, y:  1 },
        3 => Pt { x: -1, y:  1 },
        4 => Pt { x: -1, y:  0 },
        5 => Pt { x: -1, y: -1 },
        6 => Pt { x:  0, y: -1 },
        7 => Pt { x:  1, y: -1 },
        _ => panic!()
    }
}

pub fn rotate(v :Vc, a :i8) -> Vc {
    angle_v(modu(v_angle(v) + a, 8))
}

pub fn modu(a :i8, b:i8) -> i8 { (a % b + b ) % b }

