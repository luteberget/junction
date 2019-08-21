use log::*;
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


pub fn convert_railplot(railml :railmlio::topo::Topological) 
    -> Result<railplotlib::model::SchematicGraph<()>, ImportState> {

        unimplemented!()

}

pub fn convert_junction(plot :railplotlib::solvers::SchematicOutput<()>) -> Result<Model, ImportState> {

    unimplemented!()

}

