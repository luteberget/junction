pub enum LintIssue {
    RedundantSignal(EntityId),
    RedundantDetector(EntityId),
    FasterSignalPosition(EntityId, f32),
    FasterDetectorPosition(EntityId, f32),
    FasterNewSignal(EntityId, f32, Side),
    FasterNewDetector(EntityId, f32, Side),
}

pub struct LintInput {
    planning_steps: usize,
}

pub fn lint(inf :&inf, usages :&[Usage], dispatches: &Dispatch) -> LintIssue {
    let mut issues = Vec::new();


    // Lint 1: join_routes

    // combinatorially check if the number of signals (elementary routes) or detectors (partial
    // routes) can be eliminated.
    //
    // Works by composing all usages into a single SAT instance having the same number of steps
    // as the maximum of the planning steps used for each of them individually.
    //
    //  Modified plan():
    //    A. new_lit: signal_used, is false if two consecutive partial routes separated by a signal 
    //       are always activated together:  
    //         equal_or(signal_used, did_activate(r1_last), did_activate(r2_first))
    //       this constraint is compatible with others.
    //
    //    B. new_lit: detector_used, is false if two consecutive partial routes separated by a non-signal
    //       are always released together. This requires two conditions:
    //         1. equal_or(detector_used, did_release(r1_i), did_release(r1_i+1))
    //         2. forced freeing (is_freeable) must be disabled here somehow.
    //
    // This lint works only on usages, not dispatches. It could work on dispatches but would
    // need to evaluate what it means that two signals are used "at the same time". And in 
    // general it does not make so much sense... (?)
    //
    //
    // We can choose between checking each signal/detector or to do an optimization
    // problem over the number of signals/detectors (where a relative cost on signals/detectors
    // would be needed)
    //

    // LINT1 JOIN ROUTES
    issues.extend(join_routes(inf, usages));


    // 
    // # IGNORE DISPATCHES
    // Just ignore dispatches. They might be valuable, but also too hard to explain how and why.
    // For reference: 
    //   1: dispatches have no performance *goal*, but in practice, the total time could be
    //      relevant. 
    //   2: improvement in layout could be ignored by dispatches because they have specified 
    //      times for actions. 
    //   3: optimization of dispatches themselves could be suggested. 
    //   4: goals could be introduced on dispatches. But then, why not use usages.
    // For the paper, it is easier to focus on the usages only.

    //
    // Lint 2: Move components
    //
    // Works by moving signals and detectors around within their respective domains.
    // The domains are: we don't move them past switches. So "tracks" in the railML/RTM sense.
    // If there are several components on the same track, we use the relative length since last
    // component (or the beginning of the track).
    //
    // Move components requires that we have usages *with* timing constraints.
    // The "size" of the timing constraints do not matter, and is subject to optimization.
    //
    // Discrete changes can happen as we adjust. Discrete changes are either:
    //  1. The verification fails, i.e. no plans exist anymore. This has a 
    //     conceptually infinite cost.
    //  2. The plan changes. We put a "new situation" on the stack for separate 
    //     consideration.
    // The optimization runs on separate "threads" for each different dispatch plan.
    //
    // Use simulated annealing to avoid getting stuck in local minimum.
    //
    // Like in Lint1, we can choose to optimize a single signal or detector separately,
    // or to optimize the system of all of them at once.
    //

    // Lint 3: add components
    //
    // Works by splitting a partial route in two.
    //
    //
    // Both of these lints come from the "simulated annealing" process.

    issues.extend(local_changes(inf, usages))

    issues
}



pub fn join_routes(lint :&LintInput, inf :&StaticInfrastructure) -> Vec<LintIssue> {
    let num_states = 10;
    let (problem_inf, problem_trains) = convert(inf, routes, usage);
    match optimize_signals(num_states, 3, problem_inf, problem_trains, |_| true) {
        Ok(redundant_signals) => {
            redundant_signals.into_iter().map(|s| match s {
                SignalId::ExternalId(s) => LintIssue::RedundantSignal(s),
                SignalId::Anonymous(_) => LintIssue::RedundantDetector(0), // TODO carry id of detector?
                _ => unreachable!(),
            }).collect()
        },
        Err(s) => {
            println!("ERROR: {:?}", s);
            return vec![];
        }
    }
}


pub fn local_changes(lint :&LintInput, inf :&StaticInfrastructure) -> Vec<LintIssue> {

    // Simulated annealing and population

    let max_population_groups = 10; // number of dispatches
    let max_population_group_size = 10; // number of things inside dispatches
    let 

}








