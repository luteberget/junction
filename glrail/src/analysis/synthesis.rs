use planner;
use rolling;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::infrastructure::*;
use crate::vehicle::*;
use crate::scenario::*;
use crate::analysis;
use crate::dgraph;
use route_finder;



// MAIN TODOS
// 1. SignalOptimizer
//   X a. get_signal_sets
//   X b. signals
//   X c. get all dispatches
//     d. convert to abstract_dispathces
// 2. optimize_locations
//   a. powell
//   b. brent
//   c. convert from abstract dispatches
//   d. hash consing?
// 3. add track signal
// 4. maximal design

struct AbstractDispatch {
    from :EntityId,
    to :EntityId,
    switch_positions: Vec<(EntityId, rolling_inf::SwitchPosition)>,
}

pub fn synthesis<F>(
    base_inf :Vec<Option<Entity>>, 
    usages :&[Usage], 
    vehicles :&[Vehicle], 
    test: F) 
    -> Result<Vec<Entity>,String> 
  where F : Fn(f64, &[Entity]) -> bool
{
    // first, we need to create a maximal design
    let maximal_entities = maximal_design(base_inf);

    // then we find the minimum amount of signals required
    // to dispatch all usages
    let maximal_dg = dgraph::convert_entities(&maximal_entities);
    let (maximal_routes, maximal_route_issues) = route_finder::find_routes(Default::default, &maximal_dg);
    let plan_inf_maximal = analysis::plan::convert_inf(&maximal_routes);
    let plan_usages = usages.iter().map(|u| analysis::plan::convert_usage(vehicles, u)).collect::<Vec<_>>();


    let mut opt = SignalOptimizer::new(&plan_inf_maximal, &plan_usages);
    let mut min_n_signals = None;
    'outer: while let Some(signal_set) = opt.next_signal_set() {
        // have now decided on a set of signals 
        min_n_signals = min_s_signals.unwrap_or_else(|| signal_set.signals().len());
        if signal_set.signals().len() > min_n_signals.unwrap() {
            println!("No more solutions with the lowest number of signals. Stopping now.");
            break; 
        }

        let abstract_dispatches : Vec<(&Usage, Vec<AbstractDispatch>)> = 
            signal_set.get_dispatches().zip(usages.iter()).collect();

        let mut entities = signal_set.as_entities().collect();
        let score = optimize_locations(base_inf.clone(), &mut entities, &abstract_dispatches);
        if test(score, &entities) { break 'outer; }

        // try to add signals at any track/dir
        let mut current_best_score = score;
        let mut current_best_signals = entities;
        loop {
            let (mut best_score, mut best_inf) = (None, None);
            for track in base_inf.get_tracks() {
                for dir in &[Dir::Up, Dir::Down] {
                    // TODO check that any train actually goes here
                    let new_signal_entities = entities.clone();
                    add_track_signal(track,dir,&mut new_signal_entities);
                    let score = optimize_locations(base_inf.clone(), &mut new_signal_entities, &abstract_dispatches);
                    if best_score.is_none() || (best_score.is_some() && best_score.unwrap() > score) {
                        best_score = Some(score);
                        best_inf = Some(new_signal_entities);
                    }
                }
            }

            current_best_signals = best_inf.unwrap();

            // report the solution, see if consumer is happy
            if test(score, &entities) { break 'outer; }
        }
    }
}


fn optimize_locations(base_inf :Vec<Option<Entity>>, signals :&mut Vec<Entity>, dispatches :Vec<(&Usage, Vec<AbstractDispatch>)>) -> f64 {
}

fn maximal_design(base_inf :Vec<Option<Entity>>) -> Vec<Option<Entity>> {

}

fn measure_cost(entities :&Vec<Option<Entity>>, dispatches :&Vec<Vec<AbstractDispatch>>, usages :&[Usage]) -> f64 {
    unimplemented!()
}
