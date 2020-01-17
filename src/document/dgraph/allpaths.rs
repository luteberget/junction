use matches::matches;
use rolling::input::staticinfrastructure::*;
use std::collections::HashSet;
use ordered_float::OrderedFloat;

pub type Edge = (NodeId,NodeId,OrderedFloat<f64>);
pub type Path = Vec<Edge>;

pub fn path_length(p :&[Edge]) -> f64 { 
    p.iter().map(|(_,_,l)| l.into_inner()).sum() 
}

pub fn path_tail(p :&[Edge], tail :f64) -> &[Edge] {
    let start_idx = p.iter().enumerate().rev()
        .scan(0.0, |s,(i,(_,_,l))| { *s += l.into_inner(); Some((i,*s)) })
        .find(|(_i,s)| *s >= tail);
    if let Some((i,_)) = start_idx { &p[i..] } else { p }
}

pub fn paths(inf :&StaticInfrastructure, path_length_equality_margin :f64) -> Vec<Path> {
    let mut visited :HashSet<Path> = HashSet::new();
    let mut output :Vec<Path> = Vec::new();
    let mut stack :Vec<Path> = inf.nodes.iter().filter_map(|n| {
        if matches!(n.edges, Edges::ModelBoundary) { 
            if let Edges::Single(b,l) = inf.nodes[n.other_node].edges {
                Some(vec![(n.other_node,b,OrderedFloat(l))])
            } else { None }
        } else { None } }).collect();

    while let Some(mut current_path) = stack.pop() {
        if path_length(&current_path) >= path_length_equality_margin {
            let tail = path_tail(&current_path, path_length_equality_margin);
            if visited.contains(tail) {
                output.push(current_path);
                continue;
            } else {
                visited.insert(tail.to_vec());
            }
        }

        let current_node = inf.nodes[current_path.last().unwrap().1].other_node;
        match inf.nodes[current_node].edges {
            Edges::Single(target, length) => {
                current_path.push((current_node, target, OrderedFloat(length)));
                stack.push(current_path);
            },
            Edges::Switchable(obj) => {
                if let StaticObject::Switch { right_link, left_link, .. } = inf.objects[obj] {
                    let mut new_path = path_tail(&current_path, path_length_equality_margin).to_vec();
                    new_path.push((current_node, right_link.0, OrderedFloat(right_link.1)));
                    current_path.push((current_node, left_link.0, OrderedFloat(left_link.1)));
                    stack.push(new_path); stack.push(current_path);
                } else { panic!() }
            }
            Edges::Nothing | Edges::ModelBoundary => {
                output.push(current_path);
            }
        }
    }

    output
}
