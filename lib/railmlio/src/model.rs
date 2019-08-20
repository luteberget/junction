#![allow(dead_code)]



//
// For converting:
//
//
//

pub struct Topological {
    tracks :Vec<TopoTrack>,
    nodes :Vec<TopoNode>,
}

pub struct TopoTrack {
    objects :Objects,
}

pub enum AB { A, B }
pub enum Port {
    SwTrunk, SwLeft, SwRight,
    Crossing(AB, usize),
    Single,
}

pub enum TopoNode {
    BufferStop,
    OpenEnd,
    MacroscopicNode(String),
    Switch(Side),
    Crossing(CrossingType),
}



//
//
//
// original railml model
//
//
//


pub type Id = String;
pub type IdRef = String;

pub struct RailML {
    pub infrastructure :Option<Infrastructure>,
}

pub struct Infrastructure {
    pub tracks :Vec<Track>,
}

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

pub struct Node {
    pub id :Id,
    pub pos :Position,
    pub track_end :TrackEndConnection,
}

pub enum TrackEndConnection {
    Connection(Id,IdRef),
    BufferStop, OpenEnd,
    MacroscopicNode(String),
}

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

pub enum SwitchConnectionCourse { Straight, Left, Right }
pub enum ConnectionOrientation { Incoming, Outgoing, RightAngled, Unknown, Other }

pub struct SwitchConnection {
    id :Id,
    r#ref :IdRef,
    orientation :ConnectionOrientation,
    course :Option<SwitchConnectionCourse>,
    radius :Option<f64>,
    max_speed :Option<f64>,
    passable :Option<bool>,
}

pub struct Position {
    pub offset :f64,
    pub mileage: f64,
}

pub struct Objects {
    pub signals: Vec<Signal>,
    pub balises: Vec<Balise>,
}

pub struct Signal {
    id: Id,
    pos :Position,
    name :Option<String>,
    dir :TrackDirection,
    sight :Option<f64>,
    r#type :SignalType,
}

pub enum SignalType { Main, Distant, Repeater, Combined, Shunting }
pub enum SignalFunction { Exit, Home, Blocking, Intermediate }
pub enum TrackDirection { Up, Down }

pub struct Balise {
}


