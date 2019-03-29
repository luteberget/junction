/// background updates
///
///  inf -->schematic 
///      (dynamic)
///     \--> (dgraph ->) interlocking -*-> *planning --> simulation
///                                  \-*-> *simulation
///      (static)
///     \--  static analysis --> errors

use crate::app::*;
use crate::infrastructure::*;
use crate::model::*;
use crate::schematic::*;
use crate::scenario::*;
use crate::interlocking::*;
use crate::dgraph::*;
use crate::dgraph;
use crate::schematic;
use crate::wake;
use crate::analysis::sim;
use crate::analysis::plan;

// This file should encapsulate the thread pool
use std::sync::mpsc;
use std::sync::Arc;
use threadpool::ThreadPool;
use std::collections::HashMap;


pub struct BackgroundUpdates {
    pool: ThreadPool,

    // TODO these could be merged to a single channels-over-channel which 
    // would have an enum of process types, also allowing the user interface to 
    // list ongoing processes
    // or maybe it's a vector of jobs Vec<Job>, enum Job { DrawJob(Receiver<Schematic)>, .. }
    draw_rx :Option<mpsc::Receiver<Result<Schematic, String>>>,
    il_rx :Option<mpsc::Receiver<Result<(DGraph, (Vec<Route>, HashMap<EntityId,Vec<usize>>), Vec<ConvertRouteIssue>), String>>>,
    sim_rx :HashMap<usize, mpsc::Receiver<Result<History, String>>>,
    plan_rx :HashMap<usize, mpsc::Receiver<Result<Vec<Dispatch>, String>>>,
}

impl BackgroundUpdates {
    pub fn new() -> Self {
        BackgroundUpdates { 
            pool: ThreadPool::new(2),
            draw_rx : None,
            il_rx : None,
            sim_rx : HashMap::new(),
            plan_rx : HashMap::new(),
        }
    }

    pub fn status_str(&self) -> String {
        format!("bg jobs: {}/{} ({})", 
                self.pool.active_count(),
                self.pool.max_count(),
                self.pool.queued_count())
    }

    pub fn poll_updates(&mut self, model :&mut Model) {

        if let Some(Ok(res)) = self.draw_rx.as_mut().map(|f| f.try_recv()) {
            match res {
                Ok(s) => model.schematic = Derive::Ok(s),
                Err(s) => model.schematic = Derive::Err(s),
            };
            self.draw_rx = None;
        }

        //println!("Checking for updates {:?}.",self.il_rx);
        if let Some(Ok(res)) = self.il_rx.as_mut().map(|f| f.try_recv()) {
            match res {
                Ok((dgraph,(routes, route_entity_map),issues)) => {
                    println!("RECEIVED dg {:#?}", dgraph);
                    println!("RECEIVED routes {:#?}", routes);
                    println!("RECEIVED issues {:#?}", issues);
                    model.dgraph = Derive::Ok(dgraph);
                    model.interlocking.routes = Derive::Ok(Arc::new((routes, route_entity_map)));
                    for i in 0..(model.scenarios.len()) {
                        self.invalidate_scenario(i, model);
                    }
                },
                Err(s) =>  {
                    println!("ROUTE ERR {:?}", s);
                    model.dgraph = Derive::Err(s.clone());
                    model.interlocking.routes = Derive::Err(s);
                },
            };
            self.il_rx = None;
        }
        
        for (k,v) in self.sim_rx.iter_mut() {
            if let Ok(res) = v.try_recv() {
                match res {
                    Ok(h) => {
                        println!("Received sim results.");
                        model.scenarios[*k].set_history(Derive::Ok(h));
                    },
                    Err(s) => {
                        println!("Received sim error {:?}.",s);
                        model.scenarios[*k].set_history(Derive::Err(s));
                    }
                }
            }
        }
        for (k,v) in self.plan_rx.iter_mut() {
            if let Ok(res) = v.try_recv() {
                match res {
                    Ok(d) => {
                        println!("Received plan results.");
                        model.scenarios[*k].set_usage_dispatches(Derive::Ok(d));
                    },
                    Err(s) => {
                        println!("Received plan error {:?}.",s);
                        model.scenarios[*k].set_usage_dispatches(Derive::Err(s));
                    }
                }
            }
        }
    }

    pub fn invalidate_inf(&mut self, model :&mut Model) {
        // Generate new draw + static + dynamic
        self.invalidate_schematic(model);
        self.invalidate_static(model);
        self.invalidate_dynamic(model);
    }

    pub fn invalidate_schematic(&mut self, model :&mut Model) {
        model.schematic = Derive::Wait;
        let entities = model.inf.clone();
        let (draw_tx, draw_rx) = mpsc::channel();
        self.pool.execute(move || {
            let r = schematic::solve(&entities);
            if draw_tx.send(r).is_ok() { wake(); }
        });
        self.draw_rx = Some(draw_rx);
    }

    pub fn invalidate_dynamic(&mut self, model :&mut Model) {
        println!("Invalidate dynamic");
        // Delete dgraph
        model.dgraph = Derive::Wait;
        // Delete routes
        model.interlocking.routes = Derive::Wait;
        // Delete scenarios/dispatches
        for scenario in &mut model.scenarios {
            match scenario {
                Scenario::Dispatch(Dispatch { ref mut history, ..  }) => *history = Derive::Wait,
                Scenario::Usage(_, ref mut dispatches) => *dispatches = Derive::Wait,
            }
        }

        // update dgraph and interlocking
        let (il_tx,il_rx) = mpsc::channel();
        let entities = model.inf.clone();
        self.pool.execute(move || {
            let res = dgraph::convert_entities(&entities);
            let res = res.and_then(|(dg,mut issues)| {
                let (routes,mut route_issues) = 
                    route_finder::find_routes(Default::default(), &dg.rolling_inf)
                    .map_err(|_| format!("find routes error"))?;
                    //dgraph::make_routes(&dg);
                //issues.extend(route_issues);

                // convert Vec<(Route, Vec<(NodeId,NodeId)>)> 
                //     to  (Vec<Route>, HashMap<EntityId, Vec<usize>>)
                
                let mut route_vec = Vec::new();
                let mut route_entity_map = HashMap::new();
                for (ri,(r,l)) in routes.into_iter().enumerate() {
                    route_vec.push(r);
                    for (n1,n2) in l {
                        use std::iter;
                        for n in iter::once(n1).chain(iter::once(n2)) {
                            if let Some(entity) = dg.node_ids.get_by_right(&n) {
                                route_entity_map.entry(*entity).or_insert(Vec::new())
                                    .push(ri);
                            }
                        }
                    }
                }

                Ok((dg, (route_vec, route_entity_map), route_issues))
            });

            if il_tx.send(res).is_ok() { wake(); }
        });
        self.il_rx = Some(il_rx);
    }

    pub fn invalidate_scenario(&mut self, idx :usize, model :&mut Model) {
        // TODO catch unwraps
        let dgraph = Arc::clone(&model.dgraph.get().unwrap().rolling_inf);
        // TODO unnecessary conversion into hashmap
        use std::ops::Deref;
        let routes = (model.interlocking.routes.get().unwrap()).deref().clone().0
            .into_iter().enumerate().collect::<HashMap<usize,_>>();
        // TODO arc on vehicles?
        let vehicles = model.vehicles.clone();

        // Delete dispatch history and movement dispatches.
        match &mut model.scenarios[idx] {
            Scenario::Dispatch(Dispatch { ref commands, ref mut history }) => {
                *history = Derive::Wait;

                let (sim_tx,sim_rx) = mpsc::channel();
                let cmds = commands.clone();
                self.pool.execute(move || {
                    let r = sim::get_history(&vehicles, &dgraph, &routes, &cmds);
                    if sim_tx.send(r).is_ok() { wake(); }
                });

                self.sim_rx.insert(idx, sim_rx);
            },
            Scenario::Usage(ref usage, ref mut dispatches) => {
                *dispatches = Derive::Wait;

                let (plan_tx,plan_rx) = mpsc::channel();
                let spec = usage.clone();
                self.pool.execute(move || {
                    let r = plan::get_dispatches(&vehicles, &dgraph, &routes, &spec);
                    if plan_tx.send(r).is_ok() { wake(); }
                });

                self.plan_rx.insert(idx, plan_rx);
            },
        }
    }

    pub fn invalidate_static(&mut self, model :&mut Model) {
        // ...
    }
}

