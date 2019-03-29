use planner;
use rolling;
use rolling::input::staticinfrastructure as rolling_inf;
use crate::infrastructure::*;
use crate::vehicle::*;
use crate::scenario::*;
use crate::analysis;
use crate::dgraph;
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
//   a. powell
//   b. brent
//   c. convert from abstract dispatches
//   d. hash consing?
// 3. X    add track signal
// 4. X    maximal design
//
// 5. visualization
// 6. examples
//

#[derive(Debug)]
struct AbstractDispatch {
    from :EntityId, // signal or boundary node
    to :EntityId, // signal or boundary node
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
        signals.push(Object(track, 0.5*(p1 + p2), ObjectType::Signal(dir)));
    } else {
        local_signals.sort_by_key(|x| OrderedFloat(*x));
        let lowest = local_signals[0];
        local_signals.insert(0, lowest*0.5);
        let mut diff = local_signals.iter().zip(local_signals.iter().skip(1))
            .map(|(p1,p2)| (p2-p1, 0.5*(p1+p2))).collect::<Vec<_>>();
        diff.sort_by_key(|(d,_)| OrderedFloat(-*d));
        let p = diff[0].1;
        signals.push(Object(track, p, ObjectType::Signal(dir)));
    }
}

fn get_entryexit(entity_map :&BiMap<dgraph::RollingId, EntityId>, ee :&rolling_inf::RouteEntryExit) -> EntityId {
    match ee {
        rolling_inf::RouteEntryExit::Boundary(Some(id)) => {
            let id = dgraph::RollingId::Node(*id);
            if let Some(e) = entity_map.get_by_left(&id) {
                *e
            } else { panic!() }
        },
        rolling_inf::RouteEntryExit::Signal(id) => {
            let id = dgraph::RollingId::StaticObject(*id);
            if let Some(e) = entity_map.get_by_left(&id) {
                *e
            } else { panic!() }
        },
        _ => unimplemented!()
    }
}

fn get_sw_node(entity_map :&BiMap<dgraph::RollingId, EntityId>, rolling_id :rolling_inf::ObjectId) -> NodeId {
    let id = dgraph::RollingId::StaticObject(rolling_id);
    if let Some(EntityId::Node(node_id)) = entity_map.get_by_left(&id) {
        *node_id
    } else { panic!() }
}

//fn get_obj(entity_map :&BiMap<dgraph::RollingId, EntityId>, rolling_id :rolling_inf::ObjectId) -> ObjectId {
//    let id = dgraph::RollingId::StaticObject(*rolling_id);
//    if let Some(EntityId::Object(object_id)) = entity_map.get_by_left(&id) {
//        *object_id
//    } else { panic!() }
//}

fn mk_abstract_dispatch(routes :&rolling_inf::Routes<usize>,
                        entity_map :&BiMap<dgraph::RollingId, EntityId>,
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
        for (train_id,route_ids) in trains {
            let mut start :HashSet<_> = route_ids.iter().map(|rid| routes[rid].entry).collect();
            let mut end :HashSet<_> = route_ids.iter().map(|rid| routes[rid].exit).collect();
            let mut switches :BTreeSet<(_,_)> = BTreeSet::new();
            for rid in route_ids {
                start.remove(&routes[&rid].exit);
                end.remove(&routes[&rid].entry);
                for x in routes[&rid].resources.switch_positions.iter()
                                .map(|(sw,side)| (get_sw_node(entity_map, *sw), *side)) {
                                    switches.insert(x);
                                }
            }
            assert_eq!(start.len(), 1);
            assert_eq!(end.len(), 1);

            let start = get_entryexit(entity_map, start.iter().nth(0).unwrap());
            let end = get_entryexit(entity_map, end.iter().nth(0).unwrap());
            output.push(AbstractDispatch {
                from: start,
                to: end,
                train: train_id,
                switch_positions: switches,
            });
        }

    }
    output
}

fn convert_signals(maximal_inf :&Infrastructure, entity_map :&BiMap<dgraph::RollingId,EntityId>, signals :&HashMap<planner::input::SignalId, bool>) -> Vec<Object> {
    maximal_inf.iter_objects().filter(|(object_id, Object(t,p,o))| {
        if let ObjectType::Signal(_) = o {
            if let Some(dgraph::RollingId::StaticObject(dgobj)) 
                    = entity_map.get_by_right(&EntityId::Object(*object_id)) {
                *signals.get(&planner::input::SignalId::ExternalId(*dgobj)).unwrap_or(&false)
            } else { false }
        } else {
            true
        } })
    .map(|(_,o)| o.clone()).collect()
}

pub fn add_maximal(base_inf :&mut Infrastructure) {
    let maximal_signals = maximal_design(base_inf);
    for o in &maximal_signals { base_inf.new_object(o.clone()); }
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
    let maximal_signals = maximal_design(base_inf);
    let mut maximal_inf = base_inf.clone();
    for o in &maximal_signals { maximal_inf.new_object(o.clone()); }

    // then we find the minimum amount of signals required
    // to dispatch all usages
    let (maximal_dg, dg_convert_issues) = dgraph::convert_entities(&maximal_inf).unwrap();
    let (maximal_routes, maximal_route_issues) = route_finder::find_routes(Default::default(), 
                                                                           &maximal_dg.rolling_inf).unwrap();
    let maximal_routes : HashMap<usize, rolling_inf::Route> = maximal_routes.into_iter()
        .map(|(route,path)| route)
        .enumerate().collect();
    let plan_inf_maximal = analysis::plan::convert_inf(&maximal_routes);
    let plan_usages = usages.iter().map(|u| analysis::plan::convert_usage(vehicles, u)).collect::<Vec<_>>();


    let mut opt = planner::solver::SignalOptimizer::new(&plan_inf_maximal, &plan_usages);
    let mut min_n_signals = None;
    let mut current_best_signals = maximal_signals;
    'outer: while let Some(mut signal_set) = opt.next_signal_set() {
        // have now decided on a set of signals 
        let count = |s :&HashMap<planner::input::SignalId,bool>| { s.iter().filter(|(s,active)| **active).count() };
        min_n_signals = Some(min_n_signals.unwrap_or_else(|| count(signal_set.get_signals())));
        if count(signal_set.get_signals()) > min_n_signals.unwrap() {
            println!("No more solutions with the lowest number of signals. Stopping now.");
            break; 
        }

        debug!("Got a signal set with {:?} signals {:?}", min_n_signals, signal_set.get_signals());

        let mut abstract_dispatches : Vec<(&Usage, Vec<Vec<AbstractDispatch>>)> = Vec::new();
        for (i,dispatches) in signal_set.get_dispatches().iter().enumerate() {
            let usage = &usages[i];
            let abstracts = dispatches.iter().map(|d| {
                mk_abstract_dispatch(&maximal_routes, &maximal_dg.entity_names, usage, d) });
            abstract_dispatches.push((usage, abstracts.collect()));
        }
        debug!("Abstract dispatches {:#?}", abstract_dispatches);


        let mut objects = convert_signals(&maximal_inf, &maximal_dg.entity_names, signal_set.get_signals());
        debug!("Added objects {:?}", objects);

        let score = optimize_locations(&base_inf, &mut objects, &abstract_dispatches);
        if test(score, &objects) { break 'outer; }

        // try to add signals at any track/dir
        let mut current_best_score = score;
        current_best_signals = objects;
        loop {
            let (mut best_score, mut best_inf) = (None, None);
            for (track_id,_) in base_inf.iter_tracks() {
                for dir in &[Dir::Up, Dir::Down] {
                    // TODO check that any train actually goes here
                    let mut new_signal_entities = current_best_signals.clone();
                    add_track_signal(&base_inf, track_id,*dir,&mut new_signal_entities);
                    let score = optimize_locations(&base_inf, &mut new_signal_entities, &abstract_dispatches);
                    if best_score.is_none() || (best_score.is_some() && best_score.unwrap() > score) {
                        best_score = Some(score);
                        best_inf = Some(new_signal_entities);
                    }
                }
            }

            current_best_signals = best_inf.unwrap();

            // report the solution, see if consumer is happy
            if test(score, &current_best_signals) { break 'outer; }
        }
    }

    Ok(current_best_signals)
}


fn optimize_locations(base_inf :&Infrastructure, signals :&mut Vec<Object>, 
                      dispatches :&[(&Usage, Vec<Vec<AbstractDispatch>>)]) -> f64 {
    debug!("Starting optimize_locations");
    unimplemented!()
}

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
}


fn maximal_design(base_inf :&Infrastructure) -> Vec<Object> {
    let stock_length = 10.0;
    let fouling_length = 50.0;
    let overlap_lengths = vec![0.0, 150.0];

    // for ewach switch
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

fn measure_cost(inf :&Infrastructure, dispatches :&Vec<Vec<AbstractDispatch>>, usages :&[Usage]) -> f64 {
    unimplemented!()
}
