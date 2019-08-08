use std::collections::{HashMap, HashSet};
use matches::matches;
use ordered_float::OrderedFloat;
use rolling::input::staticinfrastructure as rolling_inf;
use petgraph::unionfind::UnionFind;
use crate::dgraph::*;
//use crate::model::*;
use rolling_inf::*;


// try using lapack
// least squares routine
// dgels

// pub fn test_lsq() {
// 
//     let edges = vec![(0,1,50.0),(1,2,100.0),(1,2,150.0),(2,3,50.0)];
// 
//     let rows = edges.len();
//     let columns = 4;
//     // matrix is row major cols*rows
//     let mut matrix = vec![0.0;(rows+1)*columns];
//     let mut lengths = vec![0.0;rows + 1];
// 
//     let mut order = HashMap::new();
//     order.insert(0,0);
//     order.insert(1,1);
//     order.insert(2,2);
//     order.insert(3,3);
//     //let order = order_nodes(inf);
//     for (i,(a,b,d)) in edges.iter().enumerate() {
//         let (a,b) = if order[a] <= order[b] { (a,b) } else { (b,a) };
//         matrix[i*columns + a] = -1.;
//         matrix[i*columns + b] =  1.;
//         lengths[i] = *d;
//     }
// 
//     // last one is anchoring
//     matrix[rows*columns + 0] = 1.;
// 
//     // least squares solution to Ax=b
//     let retval = unsafe {
//         lapack::c::dgels(
//             lapack::c::Layout::RowMajor, 
//             b'N', // array is not transposed
//             rows as i32, columns as i32, // array shape
//             1, // only one column in right-hand side (b)
//             &mut matrix,
//             columns as _, // stride 
//             &mut lengths,
//             1, // only one column in right-hand side (b)
//             )
//     };
// 
// 
//     println!("TEST LAPACK {:?}", matrix);
//     println!("TEST LAPACK {:?}", lengths);
// 
// 
//     if retval != 0 { panic!(); }
// }
// 



fn avg_path_length(inf :&StaticInfrastructure, from :NodeId, to :NodeId) -> f64 {
    let mut visited = HashSet::new();
    let mut stack = vec![(from,0.0)];
    let mut sum = 0.0; let mut num = 0;
    while let Some((node,dist)) = stack.pop() {
        if visited.contains(&node) { continue; }
        visited.insert(node);
        if node == to { sum += dist; num += 1; continue; }
        match inf.nodes[node].edges {
            rolling_inf::Edges::Single(b,d) => { stack.push((b,dist+d)); },
            rolling_inf::Edges::Switchable(obj) => {
                if let rolling_inf::StaticObject::Switch { right_link, left_link, .. } = inf.objects[obj] {
                    stack.push((right_link.0, dist + right_link.1));
                    stack.push((left_link.0, dist + left_link.1));
                } else { panic!() }
           },
           _ => {},
        };
    }
    sum / (num as f64)
}

pub fn auto(inf :&StaticInfrastructure) -> HashMap<NodeId,f64> {
    let mut boundaries : HashSet<NodeId> = inf.nodes.iter().enumerate().filter_map(|(i,n)| {
        if matches!(n.edges, Edges::ModelBoundary) { Some(i) } else { None }
    }).collect();
    let mut km = HashMap::new();
    while let Some(Some(boundary)) = boundaries.iter().cloned().next().map(|k| boundaries.take(&k)) {
        let mut stack = vec![(boundary, 0.0, -1)];
        while let Some((node,pos,dir)) = stack.pop() {
            if km.get(&node).is_some() { continue; }
            km.insert(node,pos);
            stack.push((inf.nodes[node].other_node, pos, -1*dir));
            if matches!(inf.nodes[node].edges, Edges::ModelBoundary) { boundaries.remove(&node); }
            match inf.nodes[node].edges { 
                rolling_inf::Edges::Single(b,d) => { 
                    stack.push((b, pos + (dir as f64) * avg_path_length(inf,node,b), -dir));
               },
               rolling_inf::Edges::Switchable(obj) => {
                    if let rolling_inf::StaticObject::Switch { right_link, left_link, .. } = inf.objects[obj] {
                        stack.push((right_link.0, pos + (dir as f64)*right_link.1, -dir));
                        stack.push((left_link.0, pos + (dir as f64)*left_link.1, -dir));
                    } else { panic!() }
               },
               _ => {},
            };
        }
    }
    km
}


