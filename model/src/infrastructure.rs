use serde::{Serialize, Deserialize};
use generational_arena::*;

#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub struct TrackId(Index);
#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub struct NodeId(Index);
#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub struct ObjectId(Index);

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct Infrastructure {
    pub tracks :Arena<Track>,
    pub nodes :Arena<Node>,
    pub objects :Arena<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Copy,Clone,PartialEq, Eq, Hash, Debug,PartialOrd,Ord)]
pub enum EntityId {
    Track(TrackId),
    Node(NodeId),
    Object(ObjectId),
}

impl Infrastructure {
    pub fn new_empty() -> Self {
        Infrastructure {
            tracks :Arena::new(),
            nodes :Arena::new(),
            objects :Arena::new(),
        }
    }

    pub fn num_entities(&self) -> usize {
        self.tracks.len() + self.nodes.len() + self.objects.len()
    }

    pub fn new_track(&mut self, t :Track) -> TrackId {
        TrackId(self.tracks.insert(t))
    }
    pub fn new_node(&mut self, t :Node) -> NodeId {
        NodeId(self.nodes.insert(t))
    }
    pub fn new_object(&mut self, t :Object) -> ObjectId {
        ObjectId(self.objects.insert(t))
    }
    pub fn track_pos_interval(&self, t :TrackId) -> Option<(f32,f32)> {
        let Track { start_node, end_node, .. } = self.get_track(&t)?;
        let Node(p1,_) = self.get_node(&start_node.0)?;
        let Node(p2,_) = self.get_node(&end_node.0)?;
        Some((*p1,*p2))
    }

    pub fn delete(&mut self, e :EntityId) {
        match e {
            EntityId::Track(TrackId(n)) => { self.tracks.remove(n); },
            EntityId::Node(NodeId(n)) => { self.nodes.remove(n); },
            EntityId::Object(ObjectId(n)) => { self.objects.remove(n); },
        }
    }
    pub fn iter_tracks(&self) -> impl Iterator<Item = (TrackId, &Track)> {
        self.tracks.iter().map(|(k,v)| (TrackId(k), v))
    }
    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter().map(|(k,v)| (NodeId(k), v))
    }
    pub fn iter_objects(&self) -> impl Iterator<Item = (ObjectId, &Object)> {
        self.objects.iter().map(|(k,v)| (ObjectId(k), v))
    }

    pub fn get_track(&self, &TrackId(x) :&TrackId) -> Option<&Track> {
        self.tracks.get(x)
    }
    pub fn get_node(&self, &NodeId(x) :&NodeId) -> Option<&Node> {
        self.nodes.get(x)
    }
    pub fn get_object(&self, &ObjectId(x) :&ObjectId) -> Option<&Object> {
        self.objects.get(x)
    }
    pub fn get_track_mut(&mut self, &TrackId(x) :&TrackId) -> Option<&mut Track> {
        self.tracks.get_mut(x)
    }
    pub fn get_node_mut(&mut self, &NodeId(x) :&NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(x)
    }
    pub fn get_object_mut(&mut self, &ObjectId(x) :&ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(x)
    }
}



pub type Pos = f32;

pub enum InfrastructureEdit {
    /// Add a new track stretching from Pos to Pos. The track makes a new component.
    NewTrack(Pos,Pos),
    /// Split a track at Pos, inserting a new node with tracks connected to open ends.
    InsertNode(TrackId, Pos, NodeType, f32),
    /// Join two two-port nodes.
    JoinNodes(NodeId, NodeId),
    /// Extend a track by moving its end node forward. There must be enough 
    /// linear space before/after the node.
    ExtendTrack(NodeId, f32),
    /// Insert an object onto a track at a given position.
    InsertObject(TrackId, Pos, ObjectType),
    /// Update entity
    ToggleBufferMacro(NodeId),
    /// Just mark the infrastructure dirty so depentents will update
    Invalidate,
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct Object(pub TrackId, pub Pos, pub ObjectType);

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub enum ObjectType {
    Signal(Dir),
    Sight {
        dir :Dir,
        signal :ObjectId,
        distance :f64,
    },
    Balise(bool),
    Detector,
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct Track {
    pub start_node: (NodeId,Port),
    pub end_node: (NodeId,Port),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[derive(Serialize, Deserialize)]
pub struct Port {
    pub dir: Dir, // Up = pointing outwards from the node, Down = inwards
    pub course: Option<Side>, // None = trunk/begin/end, Some(Left) = Left switch/crossing
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[derive(Serialize, Deserialize)]
pub enum Dir { Up, Down }

impl Dir {
    pub fn opposite(&self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
        }
    }
    pub fn factor(&self) -> isize {
        match self {
            Dir::Up => 1,
            Dir::Down => -1,
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[derive(Serialize, Deserialize)]
pub enum Side { Left, Right }

impl Side {
    pub fn other(&self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct Node(pub Pos, pub NodeType);

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub enum NodeType {
    Switch(Dir,Side),
    Crossing,
    BufferStop,
    Macro(Option<String>),
}

impl NodeType {
    pub fn num_ports(&self) -> usize {
        match self {
            NodeType::Switch (_,_) => 3,
            NodeType::Crossing => 4,
            NodeType::BufferStop | NodeType::Macro(_) => 1,
        }
    }
}

