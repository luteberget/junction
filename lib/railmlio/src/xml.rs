use crate::model::*;
use roxmltree as xml;
type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;


pub fn parse_railml(data :&str) -> BoxResult<RailML> {
    let doc = roxmltree::Document::parse(data)?;
    parse_railml_xml(&doc.root_element())
}

fn parse_railml_xml(root :&xml::Node) -> BoxResult<RailML> {
    Ok(RailML {
        infrastructure: match root.children().find(|c| c.has_tag_name("infrastructure")) {
            Some(inf) => Some(parse_infrastructure(&inf).map_err(|e| format!("{:?}", e))?),
            None => None,
        },
    })
}

fn parse_infrastructure(inf :&xml::Node) -> Result<Infrastructure, DocErr> {
    let mut tracks = Vec::new();
    if let Some(ts) = inf.children().find(|c| c.has_tag_name("tracks")) {
        for t in ts.children().filter(|c| c.has_tag_name("track")) {
            tracks.push(parse_track(&t)?);
        }
    }
    Ok(Infrastructure { tracks })
}

pub type ByteOffset = usize;
#[derive(Debug)]
pub enum DocErr {
    ElementMissing(&'static str, ByteOffset),
    AttributeMissing(&'static str, ByteOffset),
    UnexpectedElement(String, ByteOffset),
    NumberError(ByteOffset),
    BoolError(ByteOffset),
    EnumErr(&'static str, ByteOffset),
}

fn parse_track(track :&xml::Node) -> Result<Track, DocErr> {
    println!("parsing track {:?}", track);
    let topo = track.children().find(|c| c.has_tag_name("trackTopology"))
        .ok_or(DocErr::ElementMissing("trackTopology", track.range().start))?;

    Ok(Track {
        id: track.attribute("id").ok_or(DocErr::AttributeMissing("id", track.range().start))?.to_string(),
        name: track.attribute("name").map(|x| x.to_string()),
        code: track.attribute("code").map(|x| x.to_string()),
        description: track.attribute("description").map(|x| x.to_string()),
        begin: parse_track_node(&topo.children().find(|c| c.has_tag_name("trackBegin"))
                                .ok_or(DocErr::ElementMissing("trackBegin", topo.range().start))?)?,
        end: parse_track_node(&topo.children().find(|c| c.has_tag_name("trackEnd"))
                                .ok_or(DocErr::ElementMissing("trackEnd", topo.range().start))?)?,
        switches: parse_switches(&topo)?,
        objects: parse_objects(track)?,
    })
}

fn parse_objects(track :&xml::Node) -> Result<Objects, DocErr> {
    Ok(Objects::empty()) // TODO
}

fn parse_switches(topo :&xml::Node) -> Result<Vec<Switch>, DocErr> {
    let mut result = Vec::new();
    if let Some(connections) = topo.children().find(|c| c.has_tag_name("connections")) {
        for conn_obj in connections.children().filter(|c| c.is_element()) {
            if conn_obj.has_tag_name("switch") {
                result.push(parse_switch(&conn_obj)?);
            } else if conn_obj.has_tag_name("crossing") {
                result.push(parse_crossing(&conn_obj)?);
            } else {
                return Err(DocErr::UnexpectedElement(format!("{:?}", conn_obj.tag_name()), conn_obj.range().start));
            }
        }
    }
    Ok(result) 
}

fn parse_switch(sw :&xml::Node) -> Result<Switch, DocErr> {
    Ok(Switch::Switch {
        id: sw.attribute("id").ok_or(DocErr::AttributeMissing("id", sw.range().start))?.to_string(),
        pos: parse_position(sw)?,
        name: sw.attribute("name").map(|x| x.to_string()),
        description: sw.attribute("description").map(|x| x.to_string()),
        length: match sw.attribute("length") {
            Some(length) => Some(length.parse::<f64>().map_err(|_e| DocErr::NumberError(sw.range().start))?),
            None => None,
        },
        connections: parse_switch_connections(sw)?,
        track_continue_course: match sw.attribute("trackContinueCourse") {
            Some(course) => Some(parse_course(course, sw.range().start)?),
            None => None,
        },
        track_continue_radius: match sw.attribute("trackContinueRadius") {
            Some(rad) => Some(rad.parse::<f64>().map_err(|_e| DocErr::NumberError(sw.range().start))?),
            None => None,
        },
    })
}

fn parse_switch_connections(sw :&xml::Node) -> Result<Vec<SwitchConnection>, DocErr> {
    let mut result = Vec::new();
    for c in sw.children().filter(|x| x.is_element() && x.has_tag_name("connection")) {
        result.push(parse_switch_connection(&c)?);
    }
    Ok(result)
}

fn parse_switch_connection(c :&xml::Node) -> Result<SwitchConnection, DocErr> {
    Ok(SwitchConnection {
        id: c.attribute("id").ok_or(DocErr::AttributeMissing("id", c.range().start))?.to_string(),
        r#ref: c.attribute("ref").ok_or(DocErr::AttributeMissing("ref", c.range().start))?.to_string(),
        orientation: parse_orientation(c.attribute("orientation").ok_or(DocErr::AttributeMissing("orientation", c.range().start))?, c.range().start)?,
        course: match c.attribute("course") {
            Some(course) => Some(parse_course(course, c.range().start)?),
            None => None,
        },
        radius: match c.attribute("radius") {
            Some(rad) => Some(rad.parse::<f64>().map_err(|_e| DocErr::NumberError(c.range().start))?),
            None => None,
        },
        max_speed: match c.attribute("maxSpeed") {
            Some(rad) => Some(rad.parse::<f64>().map_err(|_e| DocErr::NumberError(c.range().start))?),
            None => None,
        },
        passable: match c.attribute("passable") {
            Some(passable) => Some(passable.parse::<bool>().map_err(|_e| DocErr::BoolError(c.range().start))?),
            None => None,
        },

    })
}

fn parse_course(x :&str, pos :usize) -> Result<SwitchConnectionCourse, DocErr> {
    match x {
        "left" => Ok(SwitchConnectionCourse::Left),
        "right" => Ok(SwitchConnectionCourse::Right),
        "straight" => Ok(SwitchConnectionCourse::Straight),
        _ => Err(DocErr::EnumErr("left, right, straight", pos)),
    }
}

fn parse_orientation(x :&str, pos :usize) -> Result<ConnectionOrientation, DocErr> {
    match x {
        "incoming" => Ok(ConnectionOrientation::Incoming),
        "outgoing" => Ok(ConnectionOrientation::Outgoing),
        _ => Err(DocErr::EnumErr("incoming, outgoing", pos)),
    }
}

fn parse_crossing(sw :&xml::Node) -> Result<Switch, DocErr> {
    unimplemented!()
}

fn parse_track_node(node :&xml::Node) -> Result<Node, DocErr> {
    Ok(Node {
        id: node.attribute("id").ok_or(DocErr::AttributeMissing("id", node.range().start))?.to_string(),
        pos: parse_position(node)?,
        connection: parse_track_connection(node)?,
    })
}

fn parse_track_connection(node :&xml::Node) -> Result<TrackEndConnection, DocErr> {
    if let Some(e) = node.children().find(|c| c.has_tag_name("connection")) {
        let id = e.attribute("id").ok_or(DocErr::AttributeMissing("id", e.range().start))?;
        let idref = e.attribute("ref").ok_or(DocErr::AttributeMissing("ref", e.range().start))?;
        return Ok(TrackEndConnection::Connection(id.to_string(),idref.to_string()));
    }
    if let Some(e) = node.children().find(|c| c.has_tag_name("bufferStop")) {
        return Ok(TrackEndConnection::BufferStop);
    }
    if let Some(e) = node.children().find(|c| c.has_tag_name("openEnd")) {
        return Ok(TrackEndConnection::OpenEnd);
    }
    if let Some(e) = node.children().find(|c| c.has_tag_name("macroscopicNode")) {
        let id = e.attribute("id").ok_or(DocErr::AttributeMissing("id", e.range().start))?;
        return Ok(TrackEndConnection::MacroscopicNode(id.to_string()));
    }
    Err(DocErr::ElementMissing("connection or bufferStop or openEnd or macroscopicNode", node.range().start))
}

fn parse_position(node :&xml::Node) -> Result<Position, DocErr> {
    Ok(Position {
        offset: node.attribute("pos").ok_or(DocErr::AttributeMissing("pos", node.range().start))?
            .parse::<f64>().map_err(|_e| DocErr::NumberError(node.range().start))?,
        mileage: match node.attribute("absPos") {
            Some(abs_pos) => Some(abs_pos.parse::<f64>().map_err(|_e| DocErr::NumberError(node.range().start))?),
            None => None,
        },
    })
}






