use planner;
use rolling;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::infrastructure::*;
use crate::vehicle::*;
use crate::scenario::*;
use crate::analysis;
use crate::dgraph;
use crate::dgraph::DGraph;
use route_finder;
use std::collections::{HashMap,HashSet,BTreeSet};
use ordered_float::OrderedFloat;
use bimap::BiMap;
use log::*;



// MAIN TODOS
// 1. SignalOptimizer
//   X a. get_signal_sets
//   X b. signals
//   X c. get all dispatches
//   X d. convert signals from id to glrail objects
//   x e. convert to abstract_dispathces
// 2. optimize_locations
//   a. X powell
//   b. X brent
//   c. X convert from abstract dispatches
//   d. ? hash consing? or persistent data structures?
// 3. X    add track signal
// 4. X    maximal design
//
// 5. visualization
// 6. examples
//



type VarObjId = usize;

#[derive(Debug)]
enum AbstractEntryExit {
    Const(EntityId), // boundary node or signal from base infrastructure
    Variable(VarObjId), // signal from variable infrastructure
}

#[derive(Debug)]
struct AbstractDispatch {
    from :AbstractEntryExit, // var signal, const signal, or const boundary node
    to :AbstractEntryExit, // var signal, const signal, or const boundary node
    switch_positions: BTreeSet<(NodeId, rolling_inf::SwitchPosition)>, // glrail switch node
    train :usize,
}



fn add_track_signal(inf :&Infrastructure, track :TrackId, dir :Dir, signals :&mut Vec<Object>) {
    let Node(p1,_) = inf.get_node(&inf.get_track(&track).unwrap().start_node.0).unwrap();
    let Node(p2,_) = inf.get_node(&inf.get_track(&track).unwrap().end_node.0  ).unwrap();

    let mut local_signals = Vec::new();
    for Object(t,p,o) in signals.iter() {
        if let ObjectType::Signal(sig_dir) = o {
            if sig_dir == &dir {
                local_signals.push((dir.factor() as f32)*p);
            }
        }
    }
    if local_signals.len() == 0  {
        let pos = 0.5*(p1 + p2);
        signals.push(Object(track, pos, ObjectType::Signal(dir)));
        println!("Added dist={}/{} {:?}", pos-p1, p2-pos,  signals.last());
        signals.push(Object(track, pos, ObjectType::Detector));
        println!("Added dist={}/{} {:?}", pos-p1, p2-pos,  signals.last());
    } else {
        local_signals.sort_by_key(|x| OrderedFloat(*x));
        let lowest = local_signals[0];
        local_signals.insert(0, lowest*0.5);
        let mut diff = local_signals.iter().zip(local_signals.iter().skip(1))
            .map(|(p1,p2)| (p2-p1, 0.5*(p1+p2))).collect::<Vec<_>>();
        diff.sort_by_key(|(d,_)| OrderedFloat(-*d));
        let p = diff[0].1;
        let dist = diff[0].0 / 2.0;
        signals.push(Object(track, p, ObjectType::Signal(dir)));
        println!("Added dist={}/{} {:?}", dist, dist ,  signals.last());
        signals.push(Object(track, p, ObjectType::Detector));
        println!("Added dist={}/{} {:?}", dist, dist ,  signals.last());
    }
}

fn get_entryexit(node_ids :&BiMap<EntityId,rolling_inf::NodeId>, 
                 object_ids :&BiMap<EntityId, rolling_inf::ObjectId>,
                 ee :&rolling_inf::RouteEntryExit) -> EntityId {
    match ee {
        rolling_inf::RouteEntryExit::Boundary(Some(id)) => {
            if let Some(e) = node_ids.get_by_right(id) {
                *e
            } else { panic!() }
        },
        rolling_inf::RouteEntryExit::Signal(id) => {
            if let Some(e) = object_ids.get_by_right(id) {
                *e
            } else { panic!() }
        },
        _ => panic!()
    }
}

fn get_sw_node(object_ids :&BiMap<EntityId, rolling_inf::ObjectId>,
               rolling_object :rolling_inf::ObjectId) -> NodeId {
    if let Some(EntityId::Node(node_id)) = object_ids.get_by_right(&rolling_object) {
        *node_id
    } else { panic!() }
}


fn mk_abstract_dispatch(routes :&rolling_inf::Routes<usize>,
                        node_ids :&BiMap<EntityId, rolling_inf::ObjectId>,
                        object_ids :&BiMap<EntityId, rolling_inf::ObjectId>,
                        variable_objs :&BiMap<VarObjId, ObjectId>,
                        usage :&Usage,
                        routeplan :&planner::input::RoutePlan) -> Vec<AbstractDispatch> {
    let mut output = Vec::new();
    let mut last_active_routes = BTreeSet::new();
    for state in routeplan.iter() {
        let active_routes = state.iter().filter_map(|((elementary,part), train_id)| {
            if *part == 0 && train_id.is_some() {
                Some((*elementary, train_id.unwrap()))
            } else {
                None
            }
        }).collect::<BTreeSet<_>>();

        let mut trains : HashMap<usize, Vec<usize>> = HashMap::new(); // train -> elementaryroute
        for (new_route,train_id) in active_routes.difference(&last_active_routes) {
            trains.entry(*train_id).or_insert(Vec::new()).push(*new_route);
        }
        println!("TRAINS  {:?}", trains);
        for (train_id,route_ids) in trains {
            let mut start :HashSet<_> = route_ids.iter().map(|rid| ignore_trigger(routes[rid].entry))
                .collect();
            println!("train {}: starts {:?}", train_id, start);
            let mut end :HashSet<_> = route_ids.iter().map(|rid| ignore_trigger(routes[rid].exit))
                .collect();
            let mut switches :BTreeSet<(_,_)> = BTreeSet::new();
            for rid in route_ids {
                start.remove(&ignore_trigger(routes[&rid].exit));
                println!("train {}: remove {:?} from starts {:?}", train_id, &routes[&rid].exit, start);
                end.remove(&ignore_trigger(routes[&rid].entry));
                for x in routes[&rid].resources.switch_positions.iter()
                                .map(|(sw,side)| (get_sw_node(object_ids, *sw), *side)) {
                                    switches.insert(x);
                                }
            }
            assert_eq!(start.len(), 1);
            assert_eq!(end.len(), 1);

            let start = get_entryexit(node_ids, object_ids, start.iter().nth(0).unwrap());
            let end = get_entryexit(node_ids, object_ids, end.iter().nth(0).unwrap());

            let start = if let EntityId::Object(o) = start {
                if let Some(var) = variable_objs.get_by_right(&o) {
                    AbstractEntryExit::Variable(*var)
                } else {
                    AbstractEntryExit::Const(start)
                }
            } else {
                AbstractEntryExit::Const(start)
            };

            let end = if let EntityId::Object(o) = end {
                if let Some(var) = variable_objs.get_by_right(&o) {
                    AbstractEntryExit::Variable(*var)
                } else {
                    AbstractEntryExit::Const(end)
                }
            } else {
                AbstractEntryExit::Const(end)
            };


            output.push(AbstractDispatch {
                from: start,
                to: end,
                train: train_id,
                switch_positions: switches,
            });
        }

        last_active_routes = active_routes;

    }
    output
}

pub fn ignore_trigger(r :rolling_inf::RouteEntryExit) -> rolling_inf::RouteEntryExit {
    use self::rolling_inf::RouteEntryExit;
    match r {
        RouteEntryExit::SignalTrigger { signal, .. } => RouteEntryExit::Signal(signal),
        x => x,
    }
}

fn convert_signals(maximal_inf :&Infrastructure,
                   maximal_object_names :&BiMap<VarObjId,ObjectId>, 
                   node_ids :&BiMap<EntityId, rolling_inf::NodeId>, 
                   object_ids :&BiMap<EntityId, rolling_inf::ObjectId>, 
                   signals :&HashMap<planner::input::SignalId, bool>) 
    -> (Vec<Object>, HashMap<VarObjId, usize>) {

    let mut objs = Vec::new();
    let mut names = HashMap::new();

    for (object_id, o) in maximal_inf.iter_objects() {
        if let ObjectType::Signal(_) = o.2 {
            if let Some(dgobj) = object_ids.get_by_left(&EntityId::Object(object_id)) {
                let activated = *signals.get(&planner::input::SignalId::ExternalId(*dgobj))
                    .unwrap_or(&false);
                if activated {
                    // TODO what about fixed signals?
                    let name = *maximal_object_names
                        .get_by_right(&object_id).unwrap();
                    names.insert(name, objs.len());
                    objs.push(o.clone());
                }
            }
        } else {
            objs.push(o.clone());
        }
    }

    (objs,names)
}


pub fn synthesis(
    base_inf :&Infrastructure,
    usages :&[Usage], 
    vehicles :&[Vehicle],
    test : impl Fn(f64, &[Object]) -> bool)
    -> Result<Vec<Object>,String> 
{
    debug!("Starting synthesis.");
    // first, we need to create a maximal design
    let maximal_objects = maximal_design(base_inf);
    let mut maximal_inf = base_inf.clone();
    let mut maximal_object_names : BiMap<VarObjId, ObjectId> = BiMap::new();
    for (o_idx,o) in maximal_objects.iter().enumerate() { 
        let object_id = maximal_inf.new_object(o.clone());
        maximal_object_names.insert(o_idx, object_id);
    }

    // then we find the minimum amount of signals required
    // to dispatch all usages
    let (maximal_dg, dg_convert_issues) = dgraph::convert_entities(&maximal_inf).unwrap();
    let (maximal_routes, maximal_route_issues) = route_finder::find_routes(Default::default(), 
                                                                           &maximal_dg.rolling_inf).unwrap();
    let (routes,route_entity_map) = dgraph::convert_route_map(&maximal_dg,
                                                              maximal_routes);
    let routes = routes.into_iter().enumerate().collect();
    let plan_inf_maximal = analysis::plan::convert_inf(&routes);
    let plan_usages = usages.iter().map(|u| {
        analysis::plan::convert_usage(&route_entity_map, vehicles, u)
    }).collect::<Vec<_>>();


    let mut opt = planner::optimize::SignalOptimizer::new(&plan_inf_maximal, &plan_usages);
    let mut min_n_signals = None;
    let mut current_best_signals = maximal_objects;
    'outer: while let Some(mut signal_set) = opt.next_signal_set() {

        // have now decided on a set of signals 
        let count = |s :&HashMap<planner::input::SignalId,bool>| { 
            s.iter().filter(|(s,active)| **active).count() 
        };

        min_n_signals = Some(min_n_signals.unwrap_or_else(
                || count(signal_set.get_signals())));

        if count(signal_set.get_signals()) > min_n_signals.unwrap() {
            println!("No more solutions with the lowest number of signals. Stopping now.");
            break; 
        }


        debug!("Got a signal set with {:?} signals {:?}", min_n_signals, signal_set.get_signals());

        let mut abstract_dispatches : Vec<(&Usage, Vec<Vec<AbstractDispatch>>)> = Vec::new();
        for (i,dispatches) in signal_set.get_dispatches().iter().enumerate() {
            debug!("Dispatch{} {:?}", i, dispatches);
            let usage = &usages[i];
            let abstracts = dispatches.iter().map(|d| {
                mk_abstract_dispatch(&routes, 
                                     &maximal_dg.node_ids, 
                                     &maximal_dg.object_ids, 
                                     &maximal_object_names,
                                     usage, d) });
            abstract_dispatches.push((usage, abstracts.collect()));
        }
        debug!("Abstract dispatches {:#?}", abstract_dispatches);


        let (mut objects, object_ad_names) = convert_signals(&maximal_inf, 
                                                             &maximal_object_names,
                                          &maximal_dg.node_ids, 
                                          &maximal_dg.object_ids, 
                                          signal_set.get_signals());


        debug!("Added objects {:?}", objects);

        let score = optimize_locations(&base_inf, &mut objects, 
                                       &object_ad_names, 
                                       vehicles,
                                       &abstract_dispatches);
        current_best_signals = objects.clone(); // take a copy before potentially exiting
        println!("First optimization gave score {:?}", score);
        break 'outer;
        //if test(score, &objects) { break 'outer; }

        // try to add signals at any track/dir
        let mut current_best_score = score;
        current_best_signals = objects;
        loop {
            let (mut best_score, mut best_inf) = (None, None);
            for (track_id,_) in base_inf.iter_tracks() {
                for dir in &[Dir::Up, Dir::Down] {
                    // TODO check that any train actually goes here
                    let mut new_signal_entities = current_best_signals.clone();
                    println!("Adding at {:?}", (track_id, dir));
                    add_track_signal(&base_inf, track_id,*dir,&mut new_signal_entities);
                    let score = optimize_locations(&base_inf, &mut new_signal_entities, 
                                                   &object_ad_names, 
                                                   vehicles,
                                                   &abstract_dispatches);
                    if best_score.is_none() || (best_score.is_some() && best_score.unwrap() > score) {
                        best_score = Some(score);
                        best_inf = Some(new_signal_entities);
                    }
                }
            }


            if best_score.unwrap() < current_best_score - 0.1 {
                println!("SUcecssfully addded signal {} --> {}", current_best_score, best_score.unwrap());
                current_best_score = best_score.unwrap();
                current_best_signals = best_inf.unwrap();
            } else {
                println!("No signals could improve.");
                break;
            }

            // report the solution, see if consumer is happy
            //if test(score, &current_best_signals) { break 'outer; }
        }
    }

    Ok(current_best_signals)
}

//     // do a timing test
//     use timeit::*;
//     timeit!({measure_cost(base_inf, signals, signal_varids, vehicles, dispatches);});

fn optimize_locations(base_inf :&Infrastructure, signals :&mut Vec<Object>, 
                      signal_varids :&HashMap<VarObjId, usize>,
                      vehicles :&[Vehicle],
                      dispatches :&[(&Usage, Vec<Vec<AbstractDispatch>>)]) -> f64 {
    debug!("Starting optimize_locations");
    use nalgebra::DVector;
    let dimensions = signals.len();
    let mut search_vectors = (0..dimensions).map(|i| {
        let mut v :DVector<f64> = DVector::from_element(dimensions, 0.0);
        v[i] = 1.0;
        v
    }).collect::<Vec<_>>();
    use rand::{thread_rng, Rng};
    thread_rng().shuffle(&mut search_vectors);

    let tolerance = 0.001; // 1.0 seconds tolerance?
    let (baseline_value,baseline_travel) = measure_cost(base_inf, signals, signal_varids, 
                                      vehicles, dispatches);

    //let track_poss : HashMap<TrackId, 


    // permutation of signals ordered by (track,pos) 
    let mut pos_order :Vec<(_,usize)> = signals.iter_mut().enumerate()
        .map(|(i,o)| ((o.0,OrderedFloat(o.1)),i)).collect();
    pos_order.sort_by_key(|(k,_)| *k);
    let pos_order :Vec<usize> 
        = pos_order.into_iter().map(|(_,i)| i).collect();

    pub fn pos2intrinsic(min_dist :f64,
                         base_inf: &Infrastructure, order :&[usize], objs :&[Object]) -> DVector<f64> {
        let mut last_loc = None;
        let mut output = DVector::from_element(order.len(), 0.0);
        for (i,obj_i) in order.iter().enumerate() {
            let Object(t,p,_) = &objs[*obj_i];
            let (t_low,t_high) = base_inf.track_pos_interval(*t).unwrap();
            let low_pos = last_loc.iter().filter_map(|(lt,lp)| 
                                              if *lt == t { Some(*lp) } else { None })
                .nth(0).unwrap_or(t_low);

            //assert!(low_pos < t_high);
            //let low_pos = low_pos + min_dist as f32;
            //let t_high = t_high - min_dist as f32;
            //assert!(low_pos < t_high);

            // TODO avoid approaching 1.0 intrinsic coordinate, because
            // it will make any object after it have (t_high-low_pos) ~= 0.0,
            // which will cause problems.
            output[i] = (*p as f64 - low_pos as f64)/(t_high as f64 - low_pos as f64);
            last_loc = Some((t,*p));
        }
        output
    }

    pub fn intrinsic2pos(min_dist: f64,
                         base_inf: &Infrastructure, order :&[usize], objs :&[Object],
                         x :&DVector<f64>) -> Vec<Pos> {
        let mut last_loc = None;
        let mut output = Vec::new();
        for (dx, obj_i) in x.iter().zip(order.iter()) {
            let Object(t,_,_) = &objs[*obj_i];
            let (t_low,t_high) = base_inf.track_pos_interval(*t).unwrap();
            let low_pos = last_loc.iter().filter_map(|(lt,lp)| 
                                              if *lt == t { Some(*lp) } else { None })
                .nth(0).unwrap_or(t_low);

            //println!("low pos {} t high {}", low_pos, t_high);
            //assert!(low_pos < t_high);
            //let low_pos = low_pos + min_dist as f32;
            //let t_high = (low_pos+5.0).max(t_high - min_dist as f32);
            //assert!(low_pos < t_high);

            // Output is track pos in (low_pos, t_high)
            // remapped to (t_low, t_high)
            //  pos = lerp(low_pos, t_high, dx);
            let pos = low_pos + *dx as f32 * (t_high - low_pos);
            output.push(pos);
            last_loc = Some((t, pos));
        }
        output
    }

    let min_dist = 21.0;
    let mut current_pt :DVector<f64> = pos2intrinsic(min_dist, base_inf, &pos_order, &signals);
    let (mut current_cost,_ ) = std::panic::catch_unwind(|| {
             measure_cost(base_inf, signals,
                            signal_varids, vehicles,
                            dispatches)
    }).unwrap_or((std::f64::INFINITY,0.0));

        println!("powell starting.");
    'powell: loop {
        println!("powell iteration.");
        let mut new_cost = None;
        let iter_start = current_pt.clone();
        let (mut best_vector_i,mut best_vector_len) = (None,None);
        for (v_i,v) in search_vectors.iter().enumerate() {

            // find bounds for alpha s.t. x0+alpha*v stays within [0,1]^n

            let mut max_alpha = std::f64::INFINITY;
            let mut min_alpha = -std::f64::INFINITY;
            for (v0,x0) in v.iter().cloned().zip(current_pt.iter().cloned()) {
                // x0 + alpha*v0 < 1
                if v0 > 0.0 {
                    // alpha < (1-x0)/v0
                    max_alpha = max_alpha.min( (1.0-x0)/(v0) );
                } else if v0 < 0.0 {
                    // alpha > (1-x0)/v0
                    min_alpha = min_alpha.max( (1.0-x0)/(v0) );
                }

                // x0 + alpha*v0 > 0
                if v0 > 0.0 {
                    // alpha > -x0/v0
                    min_alpha = min_alpha.max( -x0 / v0 );
                } else if v0 < 0.0 {
                    // alpha < -x0/v0
                    max_alpha = max_alpha.min( -x0 / v0 );
                }
            }

            //println!("min {:?} max {:?} ", min_alpha, max_alpha);
            //println!("v {:?}", v);
            //println!("current_pt {:?}", current_pt);
            assert!(min_alpha < max_alpha);

            let (best_alpha,best_cost) = numerical_optimization::brent_minimum(|alpha| { 
                let pt = current_pt.clone() + alpha*v;
                let new_pos = intrinsic2pos(min_dist, base_inf, &pos_order, &signals, &pt);
                for (obj_i,p) in pos_order.iter().zip(new_pos.iter()) {
                        let Object(_,pos,_) = &mut signals[*obj_i];
                        *pos = *p as f32;
                }

                let (cost,travel) = std::panic::catch_unwind(|| {
                         measure_cost(base_inf, signals,
                                        signal_varids, vehicles,
                                        dispatches)
                }).unwrap_or((std::f64::INFINITY, 0.0));

                if (travel - baseline_travel).abs() > 20.0 {
                    //println!("Deviating travel length.");
                    return std::f64::INFINITY;
                }

                //println!("brent iteration with value {:?}", cost);
                cost
            }, min_alpha, 0.0, max_alpha, 32, None);

            //println!("Brent minimum {:?} {:?}", best_alpha, best_cost);
            assert!(best_cost <= baseline_value);

            if best_vector_len.is_none() || best_vector_len.unwrap() < best_alpha.abs() {
                best_vector_i = Some(v_i);
                best_vector_len = Some(best_alpha.abs());
            }

            if (best_cost - new_cost.unwrap_or(baseline_value)).abs() > 0.5 {
                println!("improved by {}",
                     (new_cost.unwrap_or(baseline_value) - best_cost) );
                new_cost = Some(best_cost);
                current_pt += best_alpha*v;
            } else {
                println!("no improvement on this search vector. {}",
                     (new_cost.unwrap_or(baseline_value) - best_cost) );
            }
        }
        
        let iter_offset = current_pt.clone() - iter_start;
        search_vectors.remove(best_vector_i.unwrap());
        search_vectors.push(iter_offset.normalize());
        //thread_rng().shuffle(&mut search_vectors);


        // termination condition
        if new_cost.is_none() || (new_cost.unwrap() - current_cost).abs() < tolerance {
            println!("powell terminating last={} next={:?}.", current_cost, new_cost);
            break;
        } else {
            println!("powell restarting.");
            current_cost = new_cost.unwrap();
        }
    }

    // TODO update input signals positions from current_pt
    let new_pos = intrinsic2pos(min_dist, base_inf, &pos_order, &signals, &current_pt);
    for (obj_i,p) in pos_order.iter().zip(new_pos.iter()) {
            let Object(_,pos,_) = &mut signals[*obj_i];
            *pos = *p as f32;
    }

    let (cost,_) = std::panic::catch_unwind(|| {
             measure_cost(base_inf, signals,
                            signal_varids, vehicles,
                            dispatches)
    }).unwrap_or((std::f64::INFINITY,0.0));

    println!("optimize_locations: found cost {:?} (baseline {})", cost, baseline_value);

    cost
}

// TODO move this to infrastructure model
#[derive(Copy,Clone,Debug)]
struct Cursor {
    track :TrackId,
    pos :Pos,
    dir :Dir,
}

impl Cursor {
    pub fn is_valid_on(&self, inf :&Infrastructure) -> bool {
        let f = |c:&Cursor| { 
            let Track { start_node, end_node } = inf.get_track(&c.track)?;
              let Node(p1, _) = inf.get_node(&start_node.0)?;
              let Node(p2, _) = inf.get_node(&end_node.0)?;
              if *p1 <= c.pos  && c.pos <= *p2 { Some(()) } else { None } };
        f(self).is_some()
    }

    pub fn advance_nonoverlapping(&self, inf :&Infrastructure, l :f32) -> Vec<(f32,Cursor)> {
        let Track { start_node, end_node } = inf.get_track(&self.track).unwrap();
        let Node(p1, n1) = inf.get_node(&start_node.0).unwrap();
        let Node(p2, n2) = inf.get_node(&end_node.0).unwrap();
        let mut cursors : Vec<(f32,Cursor)> = Vec::new();
        match self.dir {
            Dir::Up => {
                if self.pos + l < *p2 { 
                    cursors.push((l, Cursor { pos: self.pos + l, .. *self }));
                } else { 
                    // goto other side of node, and work from there.
                    let advanced_length = p2 - self.pos;
                    match n2 {
                        NodeType::Switch(Dir::Up, _) => { // facing switch
                            // split into left and right
                            for side in vec![Side::Left, Side::Right] {
                                let next = Cursor::at_port(inf, end_node.0, 
                                                           Port { dir: Dir::Up, course: Some(side)});
                                cursors.extend(next.advance_nonoverlapping(inf, l - advanced_length).into_iter()
                                               .map(|(d,c)| (advanced_length + d, c)));
                            }
                        },
                        NodeType::Switch(Dir::Down, _) | NodeType::BufferStop | NodeType::Macro(_) => { 
                            // trailing switch or model boundary
                            // Truncate since we are doing "nonoverlapping" paths

                            // TODO make sure that this cursor is on the right side of the switch
                            let epsilon = 0.0005;
                            cursors.push((advanced_length - epsilon, Cursor { pos: p2 - epsilon , .. *self} ));
                        },
                        _ => panic!(),
                    }
                }
            },
            Dir::Down => {
                if self.pos - l > *p1 { 
                    cursors.push((l, Cursor { pos: self.pos - l, .. *self }));
                } else { 
                    // goto other side of node, and work from there.
                    let advanced_length = self.pos - p1;
                    match n1 {
                        NodeType::Switch(Dir::Down, _) => { // facing switch
                            // split into left and right
                            for side in vec![Side::Left, Side::Right] {
                                let next = Cursor::at_port(inf, start_node.0, 
                                                           Port { dir: Dir::Down, course: Some(side)});
                                cursors.extend(next.advance_nonoverlapping(inf, l - advanced_length).into_iter()
                                               .map(|(d,c)| (advanced_length + d, c)));
                            }
                        },
                        NodeType::Switch(Dir::Up, _) | NodeType::BufferStop | NodeType::Macro(_) => { // trailing switch
                            // Truncate since we are doing "nonoverlapping" paths

                            let epsilon = 0.0005;
                            cursors.push((advanced_length - epsilon, Cursor { pos: p1 + epsilon, .. *self} ));
                        },
                        _ => panic!(),
                    }
                }
            },
        }

        cursors
    }

    pub fn advance_all(&self, inf :&Infrastructure, l :f32) -> Vec<Cursor> {
        let Track { start_node, end_node } = inf.get_track(&self.track).unwrap();
        let Node(p1, n1) = inf.get_node(&start_node.0).unwrap();
        let Node(p2, n2) = inf.get_node(&end_node.0).unwrap();

        let mut cursors = Vec::new();
        match self.dir {
            Dir::Up => {
                if self.pos + l < *p2 { 
                    cursors.push(Cursor { pos: self.pos + l, .. *self });
                } else { 
                    // goto other side of node, and work from there.
                    match n2 {
                        NodeType::Switch(Dir::Up, _) => { // facing switch
                            // split into left and right
                            cursors.extend(Cursor::at_port(inf, end_node.0, 
                                                           Port { dir: Dir::Up, course: Some(Side::Left) })
                                           .advance_all(inf, l - (p2 - self.pos)));
                            cursors.extend(Cursor::at_port(inf, end_node.0, 
                                                           Port { dir: Dir::Up, course: Some(Side::Right) })
                                           .advance_all(inf, l - (p2 - self.pos)));
                        },
                        NodeType::Switch(Dir::Down, _) => { // trailing switch
                            cursors.extend(Cursor::at_port(inf, end_node.0, 
                                                           Port { dir: Dir::Up, course: None })
                                           .advance_all(inf,l - (p2 - self.pos)));
                        },
                        _ => panic!(),
                    }
                }
            },
            Dir::Down => {
                if self.pos - l > *p1 { 
                    cursors.push(Cursor { pos: self.pos - l, .. *self });
                } else { 
                    // goto other side of node, and work from there.
                    match n1 {
                        NodeType::Switch(Dir::Down, _) => { // facing switch
                            // split into left and right
                            cursors.extend(Cursor::at_port(inf, start_node.0, 
                                                           Port { dir: Dir::Down, course: Some(Side::Left) })
                                           .advance_all(inf, l - (self.pos - p1)));
                            cursors.extend(Cursor::at_port(inf, start_node.0, 
                                                           Port { dir: Dir::Down, course: Some(Side::Right) })
                                           .advance_all(inf, l - (p2 - self.pos)));
                        },
                        NodeType::Switch(Dir::Up, _) => { // trailing switch
                            cursors.extend(Cursor::at_port(inf, start_node.0, 
                                                           Port { dir: Dir::Down, course: None })
                                           .advance_all(inf, l - (self.pos - p1)));
                        },
                        _ => panic!(),
                    }
                }
            },
        };

        cursors
    }

    pub fn advance_single(&self, inf :&Infrastructure, l :f32) -> Cursor {
        let Track { start_node, end_node } = inf.get_track(&self.track).unwrap();
        let Node(p1, _) = inf.get_node(&start_node.0).unwrap();
        let Node(p2, _) = inf.get_node(&end_node.0).unwrap();

        match self.dir {
            Dir::Up => {
                if self.pos + l < *p2 { Cursor { pos: self.pos + l, .. *self } }
                else {  Cursor { pos: *p2, .. *self } }
            },
            Dir::Down => {
                if self.pos - l > *p1 { Cursor { pos: self.pos - l, .. *self } }
                else {  Cursor { pos: *p1, .. *self } }
            },
        }
    }

    pub fn at_port(inf :&Infrastructure, node_id :NodeId, port :Port) -> Cursor {
        let Node(pos, node) = inf.get_node(&node_id).unwrap();
        for (track_id, Track { start_node, end_node, .. }) in inf.iter_tracks() {
            match port.dir {
                Dir::Up =>  {
                    if start_node == &(node_id, port) { 
                        return Cursor { track: track_id, pos: *pos, dir: port.dir };
                    }
                },
                Dir::Down =>  {
                    if end_node == &(node_id, port) { 
                        return Cursor { track: track_id, pos: *pos, dir: port.dir };
                    }
                },
            }
        }
        panic!()
    }

    pub fn at_pos(track :TrackId, pos :Pos, dir :Dir) -> Cursor {
        Cursor { track, pos, dir }
    }
}

// Called when conveting Infrastructure to rolling_inf::StaticInfrastructure
// TODO custom sight distance specified by each signal.
pub fn sight_objects(inf :&Infrastructure, default_sight_distance :f64) -> Vec<Object> {
    let mut objects = Vec::new();
    for (object_id, Object(t,p,o)) in inf.iter_objects() {
        if let ObjectType::Signal(dir) = o {
            let curr = Cursor::at_pos(*t,*p,dir.opposite());
            for (dist, c) in curr.advance_nonoverlapping(inf, default_sight_distance as f32) {
                objects.push(Object(c.track, c.pos, ObjectType::Sight {
                    dir: *dir, signal: object_id, distance: dist as f64
                }));
            }
        }
    }
    objects
}

enum DesignObjectType {
    SignalDetector(Dir),
    Detector,
}

struct DesignObject(TrackId,Pos,DesignObjectType); 

fn maximal_design(base_inf :&Infrastructure) -> Vec<Object> {
    let stock_length = 10.0;
    let fouling_length = 50.0;
    let overlap_lengths = vec![0.0, 150.0];

    // for each switch
    let mut objects = Vec::new();
    for (node_id, Node(pos, node)) in base_inf.iter_nodes() {
        if let NodeType::Switch(dir,_) = node {
            // add detector at stock rail
            let trunk = Port { dir: dir.opposite(), course: None };
            let stock = Cursor::at_port(&base_inf, node_id, trunk).advance_single(&base_inf, stock_length);
            objects.push(Object(stock.track, stock.pos, ObjectType::Detector));

            // add signals (and detectors) before join (trailing switch)
            let left  = Cursor::at_port(&base_inf, node_id, Port { dir: *dir, course: Some(Side::Left)  });
            let right = Cursor::at_port(&base_inf, node_id, Port { dir: *dir, course: Some(Side::Right) });

            for l in &overlap_lengths {
                for start in vec![left, right] {
                    for c in start.advance_all(&base_inf, *l + fouling_length) {
                        objects.push(Object(c.track, c.pos, ObjectType::Signal(dir.opposite())));
                        objects.push(Object(c.track, c.pos, ObjectType::Detector));
                    }
                }
            }
        }
    }

    objects 
}

fn concretize_dispatch(ad :&AbstractDispatch, 
                       signal_varids :&HashMap<VarObjId, usize>,
                       var_map :&Vec<ObjectId>,
                       routes :&HashMap<usize, rolling_inf::Route>,
                       route_entry :&HashMap<rolling_inf::RouteEntryExit, Vec<usize>>,
                       dg :&DGraph,
                       ) -> Vec<usize> {
    use self::rolling_inf::RouteEntryExit;
    let start_id = match ad.from {
        AbstractEntryExit::Variable(var) => {
            let var_no = signal_varids[&var];
            let eid = var_map[var_no];
            EntityId::Object(eid)
        },
        AbstractEntryExit::Const(id) => id,
    };
    let end_id = match ad.to {
        AbstractEntryExit::Variable(var) => {
            let var_no = signal_varids[&var];
            let eid = var_map[var_no];
            EntityId::Object(eid)
        },
        AbstractEntryExit::Const(id) => id,
    };

    let mut curr_start = match start_id {
        EntityId::Object(sig) => {
            let rsig = dg.object_ids.get_by_left(&start_id).unwrap();
            RouteEntryExit::Signal(*rsig) // rolling_inf::ObjectId
        },
        EntityId::Node(nd) => {
            let rnode = dg.node_ids.get_by_left(&start_id).unwrap();
            RouteEntryExit::Boundary(Some(*rnode))
        },
        _ => panic!(),
    };

    let mut end = match end_id {
        EntityId::Object(sig) => {
            let rsig = dg.object_ids.get_by_left(&end_id).unwrap();
            RouteEntryExit::Signal(*rsig)
        },
        EntityId::Node(nd) => {
            let rnode = dg.node_ids.get_by_left(&end_id).unwrap();
            RouteEntryExit::Boundary(Some(*rnode))
        },
        _ => panic!(),
    };

    let mut curr_sw_idx = 0;
    // Choose any route that matches with switch positions.
    let mut output = Vec::new();

    //println!("ROUTES {:#?}", route_entry);
    'ds: while curr_start != end {
        //println!("Finding route from {:?}", curr_start);
        'rs: for route_idx in route_entry[&curr_start].iter() {
            let route = &routes[route_idx];
            // check if switch matches

            let mut switches = BTreeSet::new();
            for x in route.resources.switch_positions.iter()
                .map(|(sw,side)| (get_sw_node(&dg.object_ids, *sw), *side)) {
                               switches.insert(x);
                           }

            let sw_ok = switches.difference(&ad.switch_positions).nth(0).is_none();
            if sw_ok {
                output.push(*route_idx);
                curr_start = route.exit;
                continue 'ds;
            } else {
                continue 'rs;
            }
        }
        panic!();
    }

    output 
}

fn routes_by_entry(routes :&Vec<rolling_inf::Route>) 
    -> HashMap<rolling_inf::RouteEntryExit, Vec<usize>> {
    let mut map = HashMap::new();
    for (i,r) in routes.iter().enumerate() {
        map.entry(ignore_trigger(r.entry)).or_insert(Vec::new()).push(i);
    }
    map
}

fn measure_cost(base_inf :&Infrastructure, 
                objs :&Vec<Object>,
                signal_varids :&HashMap<VarObjId, usize>,
                vehicles :&[Vehicle], 
                dispatches :&[(&Usage, Vec<Vec<AbstractDispatch>>)]) 
    -> (f64,f64) {
    //println!("Measuring cost {:?}", objs);
    let mut infrastructure = base_inf.clone();
    let mut map = Vec::new();
    for o in objs { map.push(infrastructure.new_object(o.clone())); }

    let (dg, _) = dgraph::convert_entities(&infrastructure).unwrap();
    let (dg_routes, _) = route_finder::find_routes(Default::default(), &dg.rolling_inf).unwrap();
    let (routes,_) = dgraph::convert_route_map(&dg, dg_routes);
    //let routes :Vec<_> = routes.into_iter().enumerate().collect();
    let routes_entry = routes_by_entry(&routes);
    let routes :HashMap<usize,rolling_inf::Route> = routes.into_iter().enumerate().collect();

    let mut total_cost = 0.0;
    let mut total_travel = 0.0;
    // Each usage has a set of plans (dispatches: Vec<Vec<AbstractDispatch>)
    for (usage, plans) in dispatches {
        assert!(dispatches.len() > 0);

        // measure abstract dispatch
        let mut usage_costs = 0.0;

        for dispatches in plans.iter() {
            let mut commands = Vec::new();
            for dispatch in dispatches.iter() {
                //println!("  cost on dispatch {:?}", dispatch);
                let concrete = concretize_dispatch(dispatch, 
                           signal_varids, &map, &routes, &routes_entry, &dg);
                let vehicle = usage.movements[dispatch.train].vehicle_ref;
                commands.extend(convert_commands(vehicle, &routes, concrete));
            }

            let commands = commands.into_iter().map(|c| (0.0, c)).collect::<Vec<(f32,_)>>();
            let history = analysis::sim::get_history(vehicles, &dg.rolling_inf, &routes, &commands).unwrap();
            total_travel += traveled_length(&history);
            //println!("TIME {:?} LENGTH {:?}", dispatch_time(&history), traveled_length(&history));
            usage_costs += dispatch_time(&history); 
            // TODO use timing constraints instead as cost measure
        }
        usage_costs /= dispatches.len() as f64;
        total_cost += usage_costs;
        //println!("  sum {:?}", usage_costs);
    }
    //println!(" SUM {:?}", total_cost);
    return (total_cost,total_travel);;
}

fn convert_commands(v :usize, routes :&HashMap<usize, rolling_inf::Route>, concrete :Vec<usize>) -> Vec<Command> {
    let mut cmds = Vec::new();
    for c in concrete {
        let route = &routes[&c];
        if let rolling_inf::RouteEntryExit::Boundary(_) = route.entry {
            // Train
            cmds.push(Command::Train(v,c));
        } else {
            cmds.push(Command::Route(c));
        }
    }
    cmds
}

fn dispatch_time(h :&History) -> f64 {
    use rolling::output::history::*;
    pub fn train_time<'a>(log :impl IntoIterator<Item = &'a TrainLogEvent>) -> f64 {
        let mut t = 0.0;
        for e in log {
            match e {
                TrainLogEvent::Wait(dt) => { t += dt; },
                TrainLogEvent::Move(dt, _, _) => { t += dt; },
                _ => {},
            }
        }
        t
    }


    pub fn inf_time<'a>(log :impl IntoIterator<Item = &'a InfrastructureLogEvent>) -> f64 {
        let mut t = 0.0;
        for e in log {
            match e {
                InfrastructureLogEvent::Wait(dt) => { t += dt; },
                _ => {},
            }
        }
        t
    }

    let mut max_t = inf_time(&h.inf);
    for (_,_,t) in &h.trains { max_t = max_t.max(train_time(t)); }
    max_t
}

fn traveled_length(h :&History) -> f64 {
    use rolling::output::history::*;
    use rolling::railway::dynamics::*;
    let mut l = 0.0;
    for (_,_,t) in h.trains.iter() {
        for e in t.iter() {
            match e {
                TrainLogEvent::Move(_,_,DistanceVelocity { dx, .. }) => { l += dx; },
                _ => {},
            }
        }
    }

    l
}


pub fn add_maximal(base_inf :&mut Infrastructure) {
    let maximal_signals = maximal_design(base_inf);
    for o in &maximal_signals { 
        base_inf.new_object(o.clone()); 
    }
}
