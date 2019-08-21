use std::collections::HashMap;
use log::*;
use matches::*;
use const_cstr::const_cstr;
use crate::model;
use crate::model::Model;
use crate::viewmodel::ViewModel;
use crate::file;
use std::sync::mpsc;

pub enum ImportError {
}


pub struct ImportWindow {
    pub open :bool,
    state :ImportState,
    thread :Option<mpsc::Receiver<ImportState>>,
    thread_pool :threadpool::ThreadPool,
}

impl ImportWindow {
    pub fn new(thread_pool :threadpool::ThreadPool) -> Self {
        ImportWindow {
            open: false,
            state: ImportState::ChooseFile,
            thread: None,
            thread_pool:thread_pool,
        }
    }
}

#[derive(Debug)]
pub enum ImportState {
    Ping,
    ChooseFile,
    ReadingFile,
    SourceFileError(String),
    PlotError(String),
    WaitForDrawing,
    Available(Model),
}

impl ImportWindow {
    pub fn open(&mut self) { self.open = true; }

    pub fn update(&mut self) {
        while let Some(Ok(msg)) = self.thread.as_mut().map(|rx| rx.try_recv()) {
            info!("Import window got message  from background thread: {:?}", msg);
        }
    }

    pub fn draw(&mut self, doc :&mut ViewModel) {
        use backend_glfw::imgui::*;
        unsafe {
        igBegin(const_cstr!("Import from railML file").as_ptr(), &mut self.open as _, 0 as _);

        match &self.state {
            ImportState::ChooseFile => {
                if igButton(const_cstr!("Browse for file...").as_ptr(),
                            ImVec2 { x: 120.0, y: 0.0 }) {

                    if let Some(filename) = tinyfiledialogs::open_file_dialog("Select railML file.","", None) {
                        self.background_load_file(filename);
                    }
                }
            },

            ImportState::Available(model) => {
                if igButton(const_cstr!("Import").as_ptr(), ImVec2 { x: 80.0, y: 0.0 }) {
                    *doc = ViewModel::new(
                        model::Undoable::from(model.clone()), 
                        file::FileInfo::empty(), 
                        self.thread_pool.clone()
                    );  
                    doc.fileinfo.set_unsaved();
                    self.close();
                }
            },
            _ => {}, // TODO
        }

        igEnd();
        }
    }

    pub fn background_load_file(&mut self, filename :String) {
        info!("Starting background loading of railml from file {:?}", filename);
        let (tx,rx) = mpsc::channel();
        self.thread = Some(rx);
        self.thread_pool.execute(|| { load_railml_file(filename, tx); });
    }

    pub fn close(&mut self) {
        self.open = false;
        self.state = ImportState::ChooseFile;
        self.thread = None;
    }
}

pub fn load_railml_file(filename :String, tx :mpsc::Sender<ImportState>)  {
    // outline of steps
    // 1. read file 
    // 2. convert to railml
    // 3. convert to topo
    // 4. convert to railplot model (directed topo with mileage)
    // 5. solve railplotlib
    // 6. convert to junction model (linesegments, nodes, objects/wlocations)

    let s = match std::fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(ImportState::SourceFileError(format!("Read error: {}", e)));
            return;
        }
    };
    if tx.send(ImportState::Ping).is_err() { return; }
    info!("Read file {:?}", filename);

    let parsed = match railmlio::xml::parse_railml(&s) {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(ImportState::SourceFileError(format!("Parse error: {:?}", e)));
            return;
        },
    };
    if tx.send(ImportState::Ping).is_err() { return; }
    info!("Parsed railml");

    let topomodel = match railmlio::topo::convert_railml_topo(parsed) {
        Ok(m) => m,
        Err(e) => {
            let _ = tx.send(ImportState::SourceFileError(format!("Model conversion error: {:?}", e)));
            return;
        },
    };
    if tx.send(ImportState::Ping).is_err() { return; }
    info!("Converted to topomodel");

    let plotmodel = match convert_railplot(topomodel) {
        Ok(m) => m,
        Err(e) => {
            let _ = tx.send(e);
            return;
        },
    };
    if tx.send(ImportState::Ping).is_err() { return; }
    info!("Converted to plotmodel");

    let solver = railplotlib::solvers::LevelsSatSolver {
        criteria: vec![
            railplotlib::solvers::Goal::Bends,
            railplotlib::solvers::Goal::Width,
            railplotlib::solvers::Goal::Height,
        ],
        nodes_distinct: false,
    };
    use railplotlib::solvers::SchematicSolver;


    info!("Starting solver");
    let plot = match solver.solve(plotmodel) {
        Ok(m) => m,
        Err(e) => {
            let _ = tx.send(ImportState::PlotError(format!("Plotting error: {:?}", e)));
            return;
        },
    };
    if tx.send(ImportState::Ping).is_err() { return; }

    info!("Found model");
    let model = match convert_junction(plot) {
        Ok(m) => m,
        Err(e) => {
            let _ = tx.send(e);
            return;
        },
    };

    info!("Model available");
    let _ = tx.send(ImportState::Available(model));
}


pub fn convert_railplot(topo :railmlio::topo::Topological) 
    -> Result<railplotlib::model::SchematicGraph<()>, ImportState> {

    use railmlio::topo;
    use railplotlib::model as plot;

    enum MileageMethod { 
        /// Use the absolute position / mileage information
        /// in the railML file. This requires consistency between 
        /// absPos values on all elements, and the track directions,
        /// i.e. absPos values must be increasing along the track's direction.
        FromFile,

        /// Derive the mileage information by averaging track lengths on 
        /// all paths between locations.
        Estimated,
    }

    let method = MileageMethod::Estimated;

    match method {
        MileageMethod::FromFile => {
            unimplemented!()
        },
        MileageMethod::Estimated => {
            // start from any single node
            let start_node = topo.nodes.iter().position(|n| 
                                matches!(n, topo::TopoNode::BufferStop |
                                            topo::TopoNode::OpenEnd |
                                            topo::TopoNode::MacroscopicNode)).
                ok_or(ImportState::SourceFileError(format!("No entry/exit nodes found.")))?;

            type NodeId = usize; // index into topo.nodes

            let track_connections :HashMap<(usize,topo::AB),(usize,topo::Port)> = 
                topo.connections.iter().cloned().collect();
            println!("Track connections {:?}", track_connections);
            let node_connections :HashMap<(usize,topo::Port),(usize,topo::AB)> = 
                topo.connections.iter().map(|(a,b)| (*b,*a)).collect();
            println!("Node connections {:?}", node_connections);

            let mut km0 : HashMap<NodeId, (isize, f64)> = HashMap::new();
            km0.insert(start_node,(1,0.0));
            println!("start node {:?}", start_node);
            let (start_track,start_trackend) = node_connections.get(&(start_node, topo::Port::Single))
                        .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;
            let start_l = topo.tracks[*start_track].length;
            let other_node_port = track_connections.get(&(*start_track,start_trackend.opposite()))
                .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;

            let mut stack = vec![(*other_node_port, start_l, 1)];

            while let Some(((node,port),pos,dir)) = stack.pop() {

                if let Some((node_dir,pos)) = km0.get(&node) {
                    if *node_dir != dir { 
                        return Err(ImportState::SourceFileError(format!(
        "Inconsistent directions on tracks, need to insert mileage direction change.")));
                    } else { continue; }
                }

                km0.insert(node,(dir,pos));

                for (other_port,next_dir) in port.other_ports() {
                    let dir = dir*next_dir;
                    println!("Going to  {:?}", (other_port,dir));
                    let (track_idx,end) = node_connections.get(&(node,other_port))
                        .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;
                    let l = topo.tracks[*track_idx].length;
                    println!("Track to  {:?}", (track_idx,end,l));
                    let other_node_port = track_connections.get(&(*track_idx,end.opposite()))
                        .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;
                    println!(" ... other node  {:?}", (other_node_port));

                    stack.push((*other_node_port, pos + (dir as f64)*l, dir));
                }
            }

            println!("KM0 in mileage estimation in raiml import\n{:?}", km0);

            // now we have roughly estimated mileages and have switch orientations
            // (incoming/outgoing = increasing/decreasing milage)
            // TODO add lsqr calculations with track lengths and unknown kms.

            let mut model = plot::SchematicGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
            };

            fn to_dir(dir :isize) -> plot::Dir { 
                match dir {
                    1 => plot::Dir::Up,
                    _ => plot::Dir::Down,
                }
            }

            for (node_idx,node_type) in topo.nodes.iter().enumerate() {
                let (dir,km0) = km0[&node_idx];
                model.nodes.push(plot::Node {
                    name: format!("n{}", node_idx),
                    pos: km0,
                    shape: match node_type {
                        topo::TopoNode::BufferStop | 
                        topo::TopoNode::OpenEnd | 
                        topo::TopoNode::MacroscopicNode => 
                            if dir == 1 { plot::Shape::Begin } else { plot::Shape::End },
                        topo::TopoNode::Switch(topo::Side::Left) => 
                            plot::Shape::Switch(plot::Side::Left, to_dir(dir)),
                        topo::TopoNode::Switch(topo::Side::Right) => 
                            plot::Shape::Switch(plot::Side::Right, to_dir(dir)),

                        _ => unimplemented!(),
                    }
                });
            }

            for (track_idx,_) in topo.tracks.iter().enumerate() {
                let mut na = track_connections.get(&(track_idx,topo::AB::A))
                    .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;
                let mut nb = track_connections.get(&(track_idx,topo::AB::B))
                    .ok_or(ImportState::SourceFileError(format!("Inconsistent connections.")))?;

                if model.nodes[na.0].pos > model.nodes[nb.0].pos {
                    std::mem::swap(&mut na, &mut nb);
                }

                let convert_port = |(n,p) :(usize,topo::Port)| {
                    match p {
                        topo::Port::Trunk => plot::Port::Trunk,
                        topo::Port::Left => plot::Port::Left,
                        topo::Port::Right => plot::Port::Right,
                        topo::Port::Single => if matches!(model.nodes[n].shape, plot::Shape::Begin) {
                            plot::Port::Out } else { plot::Port::In },
                        _ => unimplemented!(),
                }};

                let pa = convert_port(*na);
                let pb = convert_port(*nb);
                model.edges.push(plot::Edge {
                    a :(format!("n{}", na.0), pa),
                    b :(format!("n{}", nb.0), pb),
                    objects :Vec::new(),
                });
            }


            Ok(model)
        }
    }
}


pub fn convert_junction(plot :railplotlib::solvers::SchematicOutput<()>) -> Result<Model, ImportState> {

    unimplemented!()

}








