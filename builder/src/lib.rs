#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


// Abstract rebuild
//
pub trait Rebuildable {
    type ChangeEvent;
    fn update(&mut self, ev :ChangeEvent);
}


// Concrete rebuild
// The concrete build structure 
pub type ModelChangeEvent {
    // Sources:
    BaseInf,
    InterlockingOptions,
    Scenarios,
    SynthesisSpec,
    //  (View is excluded -- background computations should not depend on view)

    // Derived: 
    Schematic,
    DGraph,
    Interlocking,
    Dispatches,
    Synthesized,
    Issues,
}

// hand-transformed version of the sattic_threaded_update_graph below

pub fn App::background_updates(&mut self, model :&Model, ev :ModelChangeEvent) {
    match ev {
        ModelChangeEvent::BaseInfrastructure | ModelChangeEvent::SynthesisOptions => {
            self.bg_synthesized_infrastructure(model.base.inf.clone(), model.base.synopts.clone()); // Rc-s
            // Should generate ModelChangeEvent::Infrastructure
        },
        ModelChangeEvent::Infrastructure => {
            self.bg_schematic(model.base.schematicopts.clone(), model.derived.inf.clone());
            // Should generate ModelChangeEvent::Schematic, which doesn't launch any further
            // updates.
            // Should also generate ModelChangeEvent::DGraph, which launches further updates below.
        },
        ModelChangeEvent::SchematicOptions => {
            self.bg_schematic(model.base.schematicopts.clone(), model.derived.inf.clone());
        },
        ModelChangeEvent::DGraph => {
            self.bg_interlocking(model.base.ilopts.clone(), model.derived.dgraph.clone());
            self.bg_customdatasets(model.base.customdata.clone(), model.derived.dgraph.clone());
        },
        ModelChangeEvent::InterlockingOptions => {
            self.bg_interlocking(model.base.ilopts.clone(), model.derived.inf.clone());
        },
        ModelChangeEvent::CustomDataOptions => {
            self.bg_customdatasets(model.base.customdata.clone(), model.derived.dgraph.clone());
        },
        ModelChangeEvent::Scenario(idx) => {
            self.bg_plansim(model.base.scenario[idx].clone(), 
                            model.derived.dgraph.clone(), 
                            model.derived.interlocking.clone());
        },
        ModelChangeEvent::Interlocking => {
            // Changes all the scenarios
            for idx in model.base.scenario.range() {
                self.bg_plansim(model.base.scenario[idx].clone(), 
                                model.derived.dgraph.clone(), 
                                model.derived.interlocking.clone());
            }
        }
        // ...
        // each ModelChangeEvent causes some forward jobs to be put into the queue
        // these jobs can in turn generate new modelchangeevents when they have finished.
    }
}

#[static_threaded_update_graph]
pub fn background_updates(thread_pool: ThreadPool, ev :ModelChangeEvent) {
    update Infrastructure(Synthesized i :&Inf) {
    }
    
    update Synthesized(BaseInf i :&Inf, SynthOpts o :&SynthesisOpts) {
    }

    update Schematic(Infrastructure i :&Inf) {
    }

    update DGraph(Infrastructure i :&Inf) {
    }

    update Interlocking(DGraph dg :&DGraph, InterlockingOptions il :&IlOpts) {
    }

    update CustomDatasets(DGraph dg :&DGraph, CustomOptions o :&customdata::Opts) {
    }

    update Dispatch[idx](Scenario[idx] s :&Scenario, DGraph dg :&DGraph, Interlocking il :&Il) {
    }
}

trait ModelChangeEventHandler {
    fn update(&mut self, ev :ModelChangeEvent);
}

impl Rebuildable for DerivedModel {
    type ChangeEvent = ModelChangeEvent;

    // Update graph
    // inf   ----> schematic
    //        \--------> plan/sim
    // ilopts  --/ / \-> synthesis
    // veh/specs -/ /
    // synopts-----/

    fn update(&mut self, ev :ModelChangeEvent) {
        match ev {
            ModelChangeEvent::Infrastructure => {
                start_background_interlocking();
                start_background_schematic();
            },
            ModelChangeEvent::InterlockingOptions => {
                start_background_interlocking();
            },
            ModelChangeEvent::Scenarios(idx) => {
                start_background_sim_and_plan(idx);
            },
            ModelChangeEvent::SynthesisSpec => {
                start_background_synthesis();
            },
            ModelChangeEvent::Schematic => {
                // No dependencies
            },
            ModelChangeEvent::Interlocking => {
                for idx in scenarios {
                    start_background_sim_and_plan(idx);
                }
            },
            ModelChangeEvent::Dispatches => {
            },
            ModelChangeEvent::Synthesized => {
            },
        }
    }
}

