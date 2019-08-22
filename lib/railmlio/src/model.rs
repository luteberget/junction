#![allow(dead_code)]


use crate::topo::Side;



//
//
//
// original railml model
//
//
//


pub type Id = String;
pub type IdRef = String;

#[derive(Debug)]
pub struct RailML {
    pub infrastructure :Option<Infrastructure>,
}

#[derive(Debug)]
pub struct Infrastructure {
    pub tracks :Vec<Track>,
}

#[derive(Debug)]
pub struct Track {
    pub id :Id,
    pub code :Option<String>,
    pub name :Option<String>,
    pub description :Option<String>,
    pub begin: Node,
    pub end :Node,
    pub switches :Vec<Switch>,
    pub objects :Objects,
}

#[derive(Debug)]
pub struct Node {
    pub id :Id,
    pub pos :Position,
    pub connection :TrackEndConnection,
}

#[derive(Debug)]
pub enum TrackEndConnection {
    Connection(Id,IdRef),
    BufferStop, OpenEnd,
    MacroscopicNode(String),
}

#[derive(Debug)]
pub enum Switch {
    Switch {
         id :Id,
         pos :Position,
         name :Option<String>,
         description :Option<String>,
         length: Option<f64>,
         connections :Vec<SwitchConnection>,
         track_continue_course :Option<SwitchConnectionCourse>,
         track_continue_radius :Option<f64>,
    },
    Crossing {
         id :Id,
         pos :Position,

         track_continue_course :Option<SwitchConnectionCourse>,
         track_continue_radius :Option<f64>,
         normal_position :Option<SwitchConnectionCourse>,

         length: Option<f64>,
         connections: Vec<SwitchConnection>,
    },
}

#[derive(Copy,Clone)]
#[derive(Debug)]
pub enum SwitchConnectionCourse { Straight, Left, Right }

impl SwitchConnectionCourse {
    pub fn opposite(&self) -> Option<SwitchConnectionCourse> {
        match self {
            SwitchConnectionCourse::Left => Some(SwitchConnectionCourse::Right),
            SwitchConnectionCourse::Right => Some(SwitchConnectionCourse::Left),
            _ => None,
        }
    }

    pub fn to_side(&self) -> Option<Side> {
        match self {
            SwitchConnectionCourse::Left => Some(Side::Left),
            SwitchConnectionCourse::Right => Some(Side::Right),
            _ => None,
        }
    }
}


#[derive(Debug)]
pub enum ConnectionOrientation { Incoming, Outgoing, RightAngled, Unknown, Other }

#[derive(Debug)]
pub struct SwitchConnection {
    pub id :Id,
    pub r#ref :IdRef,
    pub orientation :ConnectionOrientation,
    pub course :Option<SwitchConnectionCourse>,
    pub radius :Option<f64>,
    pub max_speed :Option<f64>,
    pub passable :Option<bool>,
}

#[derive(Debug)]
pub struct Position {
    pub offset :f64,
    pub mileage :Option<f64>,
}

#[derive(Debug)]
pub struct Objects {
    pub signals: Vec<Signal>,
    pub balises: Vec<Balise>,
}

impl Objects {
    pub fn empty() -> Objects {
        Objects {
            signals :Vec::new(),
            balises :Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Signal {
    id: Id,
    pos :Position,
    name :Option<String>,
    dir :TrackDirection,
    sight :Option<f64>,
    r#type :SignalType,
}

#[derive(Debug)]
pub enum SignalType { Main, Distant, Repeater, Combined, Shunting }
#[derive(Debug)]
pub enum SignalFunction { Exit, Home, Blocking, Intermediate }
#[derive(Debug)]
pub enum TrackDirection { Up, Down }

#[derive(Debug)]
pub struct Balise {
}


