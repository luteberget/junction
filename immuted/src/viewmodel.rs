use rolling::input::staticinfrastructure as rolling_inf;
use threadpool::ThreadPool;
use std::sync::mpsc::*;
use crate::model::*;
use crate::dgraph::*;
use std::sync::Arc;
use std::collections::HashMap;
use nalgebra_glm as glm;

// TODO data

#[derive(Debug)]
pub struct Interlocking {
    pub routes: Vec<(rolling_inf::Route, Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>)>,
    pub boundary_routes: HashMap<(i32,i32), Vec<usize>>,
    pub signal_routes: HashMap<(i32,i32), Vec<usize>>,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct History {}

pub struct Derived {
    pub dgraph :Option<Arc<DGraph>>,
    pub interlocking :Option<Arc<Interlocking>>,
    pub history :Vec<Option<Arc<History>>>,

    pub locations :Arc<HashMap<Pt,(NDType,Vc)>>,
    pub tracks :Arc<Vec<(f64, (Pt,Port),(Pt,Port))>>,
}

pub struct ViewModel {
    model: Undoable<Model>,
    derived :Derived,
    thread_pool :ThreadPool,
    get_data :Option<Receiver<SetData>>,
}

#[derive(Debug)]
pub enum SetData {
    DGraph(Arc<DGraph>),
    Interlocking(Arc<Interlocking>),
    History(usize,Arc<History>),
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
                locations :Arc::new(HashMap::new()),
                tracks :Arc::new(Vec::new()),
            },
            thread_pool: thread_pool,
            get_data: None,
        }
    }

    pub fn receive(&mut self) {
        while let Some(Ok(data)) = self.get_data.as_mut().map(|r| r.try_recv()) {
            println!("Received data from background thread {:?}", data);
            match data {
                SetData::DGraph(dgraph) => { self.derived.dgraph = Some(dgraph); },
                SetData::Interlocking(il) => { self.derived.interlocking = Some(il); },
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

        let model = self.model.get().clone(); // persistent structs

        let (tracks,locs,trackobjects, node_data) = crate::topology::convert(&model, 50.0).unwrap();
        self.derived.tracks = Arc::new(tracks);
        self.derived.locations = Arc::new(locs);
        let tracks = self.derived.tracks.clone();
        let locs = self.derived.locations.clone();

        let (tx,rx) = channel();
        self.get_data = Some(rx);
        self.thread_pool.execute(move || {
            let model = model;  // move model into thread
            let tx = tx;        // move sender into thread

            //let dgraph = dgraph::calc(&model); // calc dgraph from model.
            let dgraph = DGraphBuilder::convert(&model,&tracks,&locs,&trackobjects).expect("dgraph conversion failed");
            let dgraph = Arc::new(dgraph);

            let send_ok = tx.send(SetData::DGraph(dgraph.clone()));
            if !send_ok.is_ok() { println!("job canceled after dgraph"); return; }
            // if tx fails (channel is closed), we don't need 
            // to proceed to next step. Also, there is no harm
            // in *trying* to send the data from an obsolete thread,
            // because the update function will have replaced its 
            // receiver end of the channel, so it will anyway not
            // be placed into the struct.

            //let interlocking = interlocking::calc(&dgraph); 
            let (routes,route_issues) = 
                route_finder::find_routes(Default::default(), &dgraph.rolling_inf)
                .expect("interlocking route finder failed");
            println!("FOUND routes {:?}", routes);
            let interlocking = Arc::new(Interlocking {
                routes: routes,
                boundary_routes: HashMap::new(),
                signal_routes: HashMap::new(),
            });
                // calc interlocking from dgraph
            let send_ok = tx.send(SetData::Interlocking(interlocking.clone()));
            if !send_ok.is_ok() { println!("job canceled after interlocking"); return; }

            for (i,dispatch) in model.dispatches.iter().enumerate() {
                //let history = dispatch::run(&dgraph, &interlocking, &dispatch);
                let history = Arc::new(History {});
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

    pub fn get_closest(&self, pt :PtC) -> Option<(Ref,f32)> {
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        if let Some(((p1,p2),_param,(d,_n))) = self.get_undoable().get().get_closest_lineseg(pt) {
            thing = Some(Ref::LineSeg(p1,p2));
            dist_sqr = d; 
        }

        println!("CLOSEST NODE {:?}", self.get_closest_node(pt));
        if let Some((p,d)) = self.get_closest_node(pt) {
            if d < 0.5*0.5 {
                thing = Some(Ref::Node(p));
                dist_sqr = d;
            }
        }

        thing.map(|t| (t,dist_sqr))
    }

    pub fn get_closest_node(&self, pt :PtC) -> Option<(Pt,f32)> {
        let (mut thing, mut dist_sqr) = (None, std::f32::INFINITY);
        println!("corners {:?} vs locs {:?}", corners(pt), self.get_data().locations);
        for p in corners(pt) {
            for (px,_) in self.get_data().locations.iter() {
                if &p == px {
                    let d = glm::length2(&(pt-glm::vec2(p.x as f32,p.y as f32)));
                    if d < dist_sqr {
                        thing = Some(p);
                        dist_sqr = d;
                    }
                }
            }
        }
        thing.map(|t| (t,dist_sqr))
    }
}

