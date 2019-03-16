pub struct Infrastructure {
    pub entities :Vec<Option<Entity>>,
}

impl Infrastructure {

    pub fn new_empty() -> Self {
        Infrastructure {
            entities: Vec::new(),
        }
    }


    pub fn new_entity(&mut self, ent :Entity) -> EntityId {
        let id = self.entities.len();
        self.entities.push(Some(ent));
        id
    }

    pub fn any_object(&self) -> Option<EntityId> {
        for (i,x) in self.entities.iter().enumerate() {
            if x.is_some() {
                return Some(i);
            }
        }
        None
    }

    pub fn delete(&mut self, id :EntityId) {
        match self.entities.get_mut(id) {
            Some(mut x) => *x = None,
            _ => {},
        }
    }

    pub fn get(&self, id :EntityId) -> Option<&Entity> {
        self.entities.get(id)?.as_ref()
    }
    pub fn get_track(&self, id :EntityId) -> Option<&Track> {
        if let Some(Some(Entity::Track(ref t))) = self.entities.get(id) {
            Some(t)
        } else { None }
    }
    pub fn get_track_mut(&mut self, id :EntityId) -> Option<&mut Track> {
        if let Some(Some(Entity::Track(ref mut t))) = self.entities.get_mut(id) {
            Some(t)
        } else { None }
    }
    pub fn get_node(&self, id :EntityId) -> Option<(&f32,&Node)> {
        if let Some(Some(Entity::Node(ref p,ref t))) = self.entities.get(id) {
            Some((p,t))
        } else { None }
    }

    pub fn get_node_mut(&mut self, id :EntityId) -> Option<(&mut f32,&mut Node)> {
        if let Some(Some(Entity::Node(ref mut p,ref mut t))) = self.entities.get_mut(id) {
            Some((p,t))
        } else { None }
    }
}



pub type Pos = f32;

pub enum InfrastructureEdit {
    /// Add a new track stretching from Pos to Pos. The track makes a new component.
    NewTrack(Pos,Pos),
    /// Split a track at Pos, inserting a new node with tracks connected to open ends.
    InsertNode(EntityId, Pos, Node, f32),
    /// Join two two-port nodes.
    JoinNodes(EntityId, EntityId),
    /// Extend a track by moving its end node forward. There must be enough 
    /// linear space before/after the node.
    ExtendTrack(EntityId, f32),
    /// Insert an object onto a track at a given position.
    InsertObject(EntityId, Pos, Object),
}

pub type EntityId = usize;


#[derive(Debug,Clone)]
pub enum Entity {
    Track(Track),
    Node(Pos, Node),
    Object(EntityId, Pos, Object),
}

#[derive(Debug,Clone)]
pub enum Object {
    Signal(Dir),
    Balise(bool),
    Detector,
}

#[derive(Debug,Clone)]
pub struct Track {
    pub start_node: (EntityId,Port),
    pub end_node: (EntityId,Port),
}

#[derive(Debug,Clone,Copy)]
pub struct Port {
    pub dir: Dir, // Up = pointing outwards from the node, Down = inwards
    pub course: Option<Side>, // None = trunk/begin/end, Some(Left) = Left switch/crossing
}
#[derive(Debug,Clone,Copy)]
pub enum Dir { Up, Down }
#[derive(Debug,Clone,Copy)]
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
pub enum Node {
    Switch(Dir,Side),
    Crossing,
    BufferStop,
    Macro(Option<String>),
}

impl Node {
    pub fn num_ports(&self) -> usize {
        match self {
            Node::Switch (_,_) => 3,
            Node::Crossing => 4,
            Node::BufferStop | Node::Macro(_) => 1,
        }
    }
}

