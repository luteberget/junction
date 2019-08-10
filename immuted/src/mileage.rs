use std::collections::{HashMap, HashSet};
use matches::matches;
use ordered_float::OrderedFloat;
use rolling::input::staticinfrastructure as rolling_inf;
use petgraph::unionfind::UnionFind;
use crate::dgraph::*;
//use crate::model::*;
use rolling_inf::*;

pub fn test_lsq_rs() {
     let edges :Vec<(usize,usize,f64)> = vec![(0,1,50.0),(1,2,100.0),(1,2,150.0),(2,3,50.0)];
     let mut order : HashMap<usize,usize> = HashMap::new();
     order.insert(0,0);
     order.insert(1,1);
     order.insert(2,2);
     order.insert(3,3);

     let params = lsqr::Params {
         damp: 0.0,
         rel_mat_err: 1e-2,
         rel_rhs_err: 1e-2,
         condlim :0.0,
         iterlim: 6000, 
     };

     let mut rhs = edges.iter().map(|(a,b,d)| *d)
         .chain(std::iter::once(0.)).collect::<Vec<_>>();

     // -1  1  0  0
     //  0 -1  1  0
     //  0 -1  1  0
     //  0  0  0  1
     //  1  0  0  0
     let sol = lsqr::lsqr(|msg| println!("{}", msg),
                edges.len() + 1, order.len(), 
                params,
                |prod| {
                    match prod {
                        lsqr::Product::YAddAx { x, y } => {
                            println!("YAddAx");
                            println!("x = {:?}", x);
                            println!("y = {:?}", y);
                            // compute y += A * x
                            // y contains edge adjustments(?), x contains node kms
                            // [n_e +1, n_n] * [n_n, 1] = [n_e +1, 1]
                           for (e, (a,b,d)) in edges.iter().enumerate() {
                               let (a,b) = if order[a] <= order[b] { (*a,*b) } else { (*b,*a) };
                               y[e] += x[b] - x[a];
                           }
                           y[edges.len()] += x[0]; // the last row contains only the
                           // fixed (for the moment: the first) node.
                             println!("YAddAx end");
                            println!("x = {:?}", x);
                            println!("y = {:?}", y);
                        },
                        lsqr::Product::XAddATy { x, y } => {
                            println!("XAddATy");
                            println!("x = {:?}", x);
                            println!("y = {:?}", y);
                            // compute x += A^T * y
                            // y contains something, 
                            for (e, (a,b,d)) in edges.iter().enumerate() {
                            // TODO replace order by pre-arranging the edges to be correctly directed 
                                let (a,b) = if order[a] <= order[b] { (*a,*b) } else { (*b,*a) };
                                x[a] += -1. * y[e];
                                x[b] +=  1. * y[e];
                            }
                            x[0] += y[0]; // node 0 is fixed
                            println!("XAddATy end");
                            println!("x = {:?}", x);
                            println!("y = {:?}", y);
                        },
                    }
                },
                &mut rhs);

     println!("ok? {:?}", sol);
}

pub fn test_lsq() {
     let edges = vec![(0,1,50.0),(1,2,100.0),(1,2,150.0),(2,3,50.0)];
     let mut order = HashMap::new();
     order.insert(0,0);
     order.insert(1,1);
     order.insert(2,2);
     order.insert(3,3);


     use nalgebra::*;
     let matrix = DMatrix::from_fn(edges.len() + 1, order.len(),
       |e,n| {
          if e == edges.len() {
              if n == 0 { 1.0 } else { 0.0 }
          } else {
              let (a,b,d) = edges[e];
              let (a,b) = if order[&a] <= order[&b] { (a,b) } else { (b,a) };
              if n == a { -1.0 }
              else if n == b { 1.0 }
              else { 0.0 }
          }
       });

     let lengths = DVector::from_fn(edges.len()+1, |i,_| if i == edges.len() { 0.0 } else { edges[i].2 });


     println!("Matrix {}", matrix);
     println!("Lengths {}", lengths);
     println!("Transposed {}", matrix.transpose());

     let mut xtx = matrix.transpose() * &matrix;
     println!(" xtx {}", xtx);
     println!(" determinant {}", xtx.determinant());

     if !xtx.try_inverse_mut() { panic!(); }
     println!(" inv {}", xtx);

     let xxx = xtx * matrix.transpose();
     println!(" xxx {}", xxx);

     let beta = xxx * lengths;

     println!(" beta {}", beta);
     
}

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


