
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct PtA(Pt);

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Document {
    pieces :SymSet<Pt>,
    objects :HashMap<PtA, (PtC,Object)>,
    node_data :HashMap<Pt, NDType>, 
    vehicles :Vec<Vehicle>,
    dispatches :Vec<Vec<(f32,Command)>>,
    movements :Vec<Movement>,
}

// Synthesis is another editor?! 
// Copies topology + vehicles + movements, then manipulates objects.

pub struct DerivedStuff {
    railway :Railway, // node types, track lengths, track/lineseg correspondance
    dgraph :DGraph, // map Pt<->nodeId, PtA<->objectId
    routes :Routes, // routes, map signal/routebegin PtA<->routes
    history :Vec<History>, // dispatch<->history
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    name :String,
    length: f64, 
    max_acc: f64,
    max_brk: f64,
    max_vel: f64,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Movement {
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Command {
    Train(Pt, usize), // Vehicle index
    Route(PtA, PtA, Option<usize>), // TODO this needs to refer to routes!
    // TODO Swing(PtA, usize), // Set route end point's new overlap by indexing into list of
    // alternatives
}



#[derive(Debug)]
pub struct SchematicCanvas {
    document :Document,
    tool: Tool,
    selection : (HashSet<(Pt,Pt)>, HashSet<PtA>),
    current_dispatch :Option<usize>,
    scale: Option<usize>,
    translate :Option<ImVec2>,
    adding_line :Option<Pt>,
    adding_object: Option<((f32,f32),(Pt,Pt))>,  // Pt-continuous
    editing_node: Option<Pt>,
    selecting_rectangle: Option<ImVec4>,
    dragging_objects :Option<PtC>,
}

// TODO model editor state like this
enum CurrentAction {
    None,
    DrawingLine(Pt),
    SelectObjectType(()),
    PlacingObject(PtC, (Pt,Pt)),
    EditingNode(Pt),
    EditingObject(usize),
    SelectingRectangle(ImVec4),
    DraggingObjectsDiscretely(PtC),
}
