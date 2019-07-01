use threadpool::ThreadPool;
use std::sync::mpsc::*;
use crate::model::*;

// TODO data
#[derive(Clone)]
#[derive(Debug)]
pub struct DGraph {}
#[derive(Clone)]
#[derive(Debug)]
pub struct Interlocking {}
#[derive(Clone)]
#[derive(Debug)]
pub struct History {}

pub struct Derived {
    pub dgraph :Option<DGraph>,
    pub interlocking :Option<Interlocking>,
    pub history :Vec<Option<History>>,
}

pub struct ViewModel {
    model: Undoable<Model>,
    derived :Derived,
    thread_pool :ThreadPool,
    get_data :Option<Receiver<SetData>>,
}

#[derive(Debug)]
pub enum SetData {
    DGraph(DGraph),
    Interlocking(Interlocking),
    History(usize,History),
}

impl ViewModel {
    pub fn new(model :Undoable<Model>, 
               thread_pool: ThreadPool) -> ViewModel {
        ViewModel {
            model: model,
            derived: Derived { 
                dgraph: None, 
                interlocking: None, 
                history: Vec::new(), 
            },
            thread_pool: thread_pool,
            get_data: None,
        }
    }

    pub fn receive(&mut self) {
        while let Some(Ok(data)) = self.get_data.as_mut().map(|r| r.try_recv()) {
            println!("Received data from background thread {:?}", 
                     data);
            match data {
                SetData::DGraph(dgraph) => { self.derived.dgraph = Some(dgraph); },
                _ => {},
                // ...
            }
        }

        if let Some(Err(_)) = self.get_data.as_mut().map(|r| r.try_recv()) {
            // channel was closed, discard handle
            // TODO is this necessary?
            self.get_data = None;
        }
    }

    fn update(&mut self) {
        let (tx,rx) = channel();
        self.get_data = Some(rx);
        let model = self.model.get().clone(); // persistent structs
        self.thread_pool.execute(move || {
            let model = model;  // move model into thread
            let tx = tx;        // move sender into thread

            //let dgraph = dgraph::calc(&model); // calc dgraph from model.
            let dgraph = DGraph {};
            let send_ok = tx.send(SetData::DGraph(dgraph.clone()));
            if !send_ok.is_ok() { println!("job canceled after dgraph"); return; }
            // if tx fails (channel is closed), we don't need 
            // to proceed to next step. Also, there is no harm
            // in *trying* to send the data from an obsolete thread,
            // because the update function will have replaced its 
            // receiver end of the channel, so it will anyway not
            // be placed into the struct.

            //let interlocking = interlocking::calc(&dgraph); 
            let interlocking = Interlocking {};
                // calc interlocking from dgraph
            let send_ok = tx.send(SetData::Interlocking(interlocking.clone()));
            if !send_ok.is_ok() { println!("job canceled after interlocking"); return; }

            for (i,dispatch) in model.dispatches.iter().enumerate() {
                //let history = dispatch::run(&dgraph, &interlocking, &dispatch);
                let history = History {};
                let send_ok = tx.send(SetData::History(i, history));
                if !send_ok.is_ok() { println!("job canceled after dispatch"); return; }
            }

        });
    }

    // TODO what is a better api here?
    pub fn get_undoable(&self) -> &Undoable<Model> {
        &self.model
    }

    pub fn get_data(&self) -> &Derived {
        &self.derived
    }

    pub fn set_model(&mut self, m :Model) {
        self.model.set(m);
        self.update();
    }

    pub fn undo(&mut self) {
        self.model.undo();
        self.update();
    }

    pub fn redo(&mut self) {
        self.model.redo();
        self.update();
    }
}

