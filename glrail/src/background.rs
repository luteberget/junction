/// background updates
///
///  inf --> schematic
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

// This file should encapsulate all use of futures_cpupool.
//use futures::{Future, Async};
//use futures_cpupool::{CpuPool, CpuFuture};
use std::sync::mpsc;
use threadpool::ThreadPool;


pub struct BackgroundUpdates {
    pool: ThreadPool,
    schematic_rx :Option<mpsc::Receiver<Result<Schematic, String>>>,
    il_rx :Option<mpsc::Receiver<Result<(DGraph, Vec<Route>, Vec<ConvertRouteIssue>), String>>>,
}

impl BackgroundUpdates {
    pub fn new() -> Self {
        BackgroundUpdates { 
            pool: ThreadPool::new(2),
            schematic_rx : None,
            il_rx : None,
        }
    }

    pub fn poll_updates(&mut self, model :&mut Model) {

        if let Some(Ok(res)) = self.schematic_rx.as_mut().map(|f| f.try_recv()) {
            match res {
                Ok(s) => model.schematic = Derive::Ok(s),
                Err(s) => model.schematic = Derive::Err(s),
            }
        }

        if let Some(Ok(res)) = self.il_rx.as_mut().map(|f| f.try_recv()) {
            match res {
                Ok((dgraph,routes,issues)) => {
                    println!("RECEIVED routes {:?}", routes);
                    println!("RECEIVED routes {:?}", issues);
                    model.dgraph = Derive::Ok(dgraph);
                    model.interlocking.routes = Derive::Ok(routes);
                    for i in 0..(model.scenarios.len()) {
                        self.invalidate_scenario(i, model);
                    }
                },
                Err(s) =>  {
                    model.dgraph = Derive::Err(s.clone());
                    model.interlocking.routes = Derive::Err(s);
                },
            }
        }
    }

    pub fn invalidate_inf(&mut self, model :&mut Model) {
        // Generate new schematic + static + dynamic
        self.invalidate_schematic(model);
        self.invalidate_static(model);
        self.invalidate_dynamic(model);
    }

    pub fn invalidate_schematic(&mut self, model :&mut Model) {
        model.schematic = Derive::Wait;
        let entities = model.inf.entities.clone();
        let (schematic_tx, schematic_rx) = mpsc::channel();
        self.pool.execute(move || {
            let r = schematic::solve(&entities);
            if schematic_tx.send(r).is_ok() { wake(); }
        });
        self.schematic_rx = Some(schematic_rx);
    }

    pub fn invalidate_dynamic(&mut self, model :&mut Model) {
        // Delete dgraph
        model.dgraph = Derive::Wait;
        // Delete routes
        model.interlocking.routes = Derive::Wait;
        // Delete scenarios/dispatches
        for scenario in &mut model.scenarios {
            match scenario {
                Scenario::Dispatch(Dispatch { ref mut history, ..  }) => *history = Derive::Wait,
                Scenario::Movement(_, ref mut dispatches) => *dispatches = Derive::Wait,
            }
        }

        // update dgraph and interlocking
        let (il_tx,il_rx) = mpsc::channel();
        let entities = model.inf.entities.clone();
        self.pool.execute(move || {
            let res = dgraph::convert_entities(&entities)
                .map(|dg| (dg, Vec::new())); // TODO get issues from dgraph conversion.
            let res = res.and_then(|(dg,mut issues)| {
                let (routes,mut route_issues) = dgraph::make_routes(&dg);
                issues.extend(route_issues);
                Ok((dg, routes, issues))
            });

            if il_tx.send(res).is_ok() { wake(); }
        });
        self.il_rx = Some(il_rx);
    }

    pub fn invalidate_scenario(&mut self, idx :usize, model :&mut Model) {
        // Delete dispatch history and movement dispatches.
        match &mut model.scenarios[idx] {
            Scenario::Dispatch(Dispatch { ref mut history, ..  }) => {
                *history = Derive::Wait;
            },
            Scenario::Movement(_, ref mut dispatches) => {
                *dispatches = Derive::Wait;
            },
        }
    }

    pub fn invalidate_static(&mut self, model :&mut Model) {
        // ...
    }
}

