use rolling::input::staticinfrastructure::*;
use std::collections::{HashSet, BTreeSet};
use smallvec::SmallVec;

#[derive(Debug)]
pub enum ConvertRouteIssue {
    NoBoundaries,
    StateConversionFailed( RouteEntryExit, RouteEntryExit),
    ExitedUnenteredSection( RouteEntryExit, ObjectId),
    RouteTooShort(RouteEntryExit, RouteEntryExit),
}

#[derive(Debug)]
pub enum ConvertRouteError {
    String(String),
}

pub struct Config {
    // TODO what can we put in here
    //
    // 0. section overlap minimum length
    // 1. route minimum length
    // ---
    // 2. overlap policy

    section_tolerance :f64,
    route_minimum_length: f64,
}

impl Default for Config {
    fn default() -> Config { Config {
        section_tolerance: 15.0,
        route_minimum_length: 15.0,
    } }
}

#[derive(Debug, Clone)]
struct RouteEntry {
    pub node: NodeId,
    pub entry: RouteEntryExit,
    pub section: Option<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct Path {
    node: NodeId,
    length: f64,
    entered_sections: SmallVec<[(ObjectId, f64); 2]>,
    exited_sections: SmallVec<[(ObjectId, f64, f64);4]>,
    switches: SmallVec<[(ObjectId, f64, SwitchPosition);4]>,
    edges_taken :Vec<(NodeId, NodeId)>,
}

pub type RoutePath = Vec<(NodeId,NodeId)>;

pub fn find_routes(config :Config, model :&StaticInfrastructure) -> Result<(Vec<(Route,RoutePath)>, Vec<ConvertRouteIssue>), ConvertRouteError> {

    let mut routes = Vec::new();
    let mut issues = Vec::new();

    let boundary_nodes = model.nodes.iter().enumerate()
        .filter_map(|(i,x)| if let Edges::ModelBoundary = x.edges { Some(i) } 
                    else { None });

    let mut entry_visited = HashSet::new();
    for boundary_idx in boundary_nodes {
        println!("Boundary start {:?}", boundary_idx);

        let mut entry_stack = Vec::new();
        entry_stack.push(RouteEntry {
            node: model.nodes[boundary_idx].other_node,
            entry: RouteEntryExit::Boundary(Some(boundary_idx)),
            section: None,
        });
        entry_visited.insert(model.nodes[boundary_idx].other_node);

        while entry_stack.len() > 0 {
            let entry = entry_stack.pop().unwrap();
            let mut search_stack = Vec::new();

            let mut switches_path_visited : BTreeSet<BTreeSet<(ObjectId, SwitchPosition)>> = BTreeSet::new();
            search_stack.push(Path {
                node: entry.node,
                entered_sections: entry.section.into_iter().map(|x| (x, 0.0)).collect(),
                exited_sections: SmallVec::new(),
                switches: SmallVec::new(),
                length: 0.0,
                edges_taken: vec![],
            });

            while search_stack.len() > 0 {
                let mut curr_state = search_stack.pop().unwrap();
                loop { // TODO make absolutely sure this terminates
                    let mut is_exit = false;

                    if curr_state.node != entry.node {
                        // Check what is in here
                        for obj_idx in model.nodes[curr_state.node].objects.iter() {
                            match &model.objects[*obj_idx] {
                                StaticObject::Signal => {
                                    let exit = RouteEntryExit::Signal(*obj_idx);
                                    match make_route(&config, &curr_state, entry.entry, exit) {
                                        Ok(route) => routes.push((route, curr_state.edges_taken.clone())),
                                        Err(err) => issues.push(err),
                                    }

                                    if entry_visited.insert(curr_state.node) {
                                        entry_stack.push(RouteEntry {
                                            node: curr_state.node,
                                            entry: RouteEntryExit::Signal(*obj_idx),
                                            section: curr_state.entered_sections.iter().nth(0).map(|x| x.0),
                                        });
                                    }

                                    is_exit = true;
                                },
                                StaticObject::TVDLimit { enter, exit } => {
                                    if let Some(s) = enter {
                                        curr_state.entered_sections.push((*s, curr_state.length));
                                    }
                                    if let Some(s) = exit {
                                        if let Some(i) = curr_state.entered_sections.iter().position(|y| y.0 == *s) {
                                            let e = curr_state.entered_sections.remove(i);
                                            curr_state.exited_sections.push((e.0, e.1, curr_state.length));
                                        } else {
                                            issues.push(ConvertRouteIssue::ExitedUnenteredSection(entry.entry, *s));
                                        }
                                    }
                                },
                                _ => {} // sight, switch, sections, are not relevant
                            }
                        }
                    }

                    if is_exit { break; }

                    match model.nodes[curr_state.node].edges {
                        Edges::Nothing => { break; },
                        Edges::ModelBoundary => {
                            let exit = RouteEntryExit::Boundary(Some(curr_state.node));
                            match make_route(&config, &curr_state, entry.entry, exit) {
                                Ok(route) => routes.push((route, curr_state.edges_taken.clone())),
                                Err(err) => issues.push(err),
                            }
                            break;
                        },
                        Edges::Single(other, d) => {
                            // Trailing switches: look at the outgoing edges from opposite node.
                            match model.nodes[other].edges {
                                Edges::Switchable(sw) => {
                                    if let Some(StaticObject::Switch { left_link, right_link, .. }) = model.objects.get(sw) {
                                        let pos = if left_link.0 == curr_state.node { SwitchPosition::Left } 
                                                    else if right_link.0 == curr_state.node { SwitchPosition::Right }
                                                    else {
                                                        return Err(ConvertRouteError::String(format!("Switch misconfigured {}", sw))); };
                                        curr_state.switches.push((sw,curr_state.length,pos));
                                    }  else {
                                        return Err(ConvertRouteError::String(format!("Switch misconfigured {}", sw)));
                                    }

                                },
                                _ => {},
                            };
                            curr_state.edges_taken.push((curr_state.node, other));
                            curr_state.node = model.nodes[other].other_node;
                            curr_state.length += d;
                        },
                        Edges::Switchable(sw) => {
                            if let Some(StaticObject::Switch { left_link, right_link, .. }) = model.objects.get(sw) {
                                let mut right_state = curr_state.clone();
                                let mut left_state = curr_state;

                                right_state.edges_taken.push((right_state.node, right_link.0));
                                left_state.edges_taken.push((left_state.node, left_link.0));
                                right_state.node = model.nodes[right_link.0].other_node;
                                left_state.node = model.nodes[left_link.0].other_node;
                                right_state.switches.push((sw, right_state.length, SwitchPosition::Right));
                                left_state.switches.push((sw, left_state.length, SwitchPosition::Left));
                                right_state.length += right_link.1;
                                left_state.length += left_link.1;

                                if switches_path_visited.insert(
                                    right_state.switches.iter().map(|(sw,_l,pos)| (*sw,*pos)).collect()) {
                                    search_stack.push(right_state);
                                }
                                if switches_path_visited.insert(
                                    left_state.switches.iter().map(|(sw,_l,pos)| (*sw,*pos)).collect()) {
                                    search_stack.push(left_state);
                                }

                                break;
                            } else {
                                return Err(ConvertRouteError::String(format!("Switch misconfigured {}", sw)));
                            }
                        },
                    };
                }
            }
        }
    }

    if !(entry_visited.len() > 0) {
        issues.push(ConvertRouteIssue::NoBoundaries);
    }

    // TODO
// ///        // Remove release of resources that were not aquired
// ///    for r in &mut routes {
// ///        let resources = r.sections.iter().chain(r.switches.iter().map(|&(ref sw,_)| sw)).collect::<Vec<_>>();
// ///        for &mut (_,_,ref mut res) in &mut r.releases {
// ///            res.retain(|x| resources.contains(&x));
// ///        }
// ///    }


    Ok((routes,issues))
}


pub fn make_route(config: &Config, state :&Path, entry :RouteEntryExit, exit: RouteEntryExit) -> Result<Route, ConvertRouteIssue> {
    if state.length < config.route_minimum_length {
        return Err(ConvertRouteIssue::RouteTooShort(entry, exit));
    }

    let mut sections = state.exited_sections.clone();
    sections.extend(state.entered_sections.iter().map(|&(x, l)| (x, l, state.length)));
    sections.retain(|&mut (_,a,b)| b-a > config.section_tolerance);

    let trigger = sections.first();
    let entry = match (trigger,entry) {
        (Some((tvd,_,_)),RouteEntryExit::Signal(x)) => RouteEntryExit::SignalTrigger { signal: x, trigger_section: *tvd },
        _ => entry,
    };

    let mut cleared_length = 0.0;
    let mut releases = sections.iter().map(|(trigger, start, end)| {
        let start = if cleared_length > *start { cleared_length } else { *start };
        let length = *end-start;
        cleared_length += length;
        let mut resources = vec![*trigger];
        for (sw,pos,_side) in &state.switches {
            if start <= *pos && pos < end {
                resources.push(*sw);
            }
        }
        //(trigger, length, resources)

        Release {
            trigger: *trigger,
            length: length,
            resources: resources.into(),
        }
    }).collect::<Vec<_>>();

    let sum_releases_length = releases.iter().map(|r| r.length).sum::<f64>();
    if releases.len() > 0 && sum_releases_length != state.length {
        println!("Release length and route length differ by {} {} {:?} {:?}", 
                 state.length, sum_releases_length, entry, exit);
        releases.last_mut().unwrap().length += state.length - sum_releases_length;
    } 

    Ok(Route {
        entry: entry,
        exit: exit,
        length: state.length,

        resources: RouteResources {
            sections: sections.into_iter().map(|(x,_,_)| x).collect(),
            switch_positions: state.switches.iter().map(|(x,_,s)| (*x,*s)).collect(),
            releases: releases.into(),
        },

        overlaps: SmallVec::new(),
        swinging_overlap: false,
    })
}

