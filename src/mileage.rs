use std::collections::{HashMap, HashSet};
use matches::matches;
use rolling::input::staticinfrastructure::*;
use petgraph::unionfind::UnionFind;
use crate::model::Pt;


fn take_boundary(node_ids :&HashMap<NodeId, Pt>, boundaries :&mut HashSet<NodeId>) -> Option<NodeId> {
    if let Some(id) = boundaries.iter().min_by_key(|n| { node_ids.get(n).map(|pt| (pt.x,-pt.y)).unwrap_or((10000,0)) }) {
        let id = *id;
        boundaries.take(&id)
    } else { None }
}

pub fn auto(node_ids :&HashMap<NodeId,Pt>, inf :&StaticInfrastructure) -> HashMap<NodeId, f64> {
    let mut boundaries : HashSet<NodeId> = inf.nodes.iter().enumerate().filter_map(|(i,n)| {
        if matches!(n.edges, Edges::ModelBoundary) { Some(i) } else { None } }).collect();
    // TODO select leftmost boundaries first
    // TODO match unconnected components' km by x coordinate?

    let mut edges : Vec<(NodeId,NodeId,f64)> = Vec::new();
    let mut km0 : HashMap<NodeId,f64> = HashMap::new();
    let mut uf = UnionFind::new(inf.nodes.len());
    let mut fixed = Vec::new();
    while let Some(boundary) = take_boundary(node_ids, &mut boundaries) {
        fixed.push(boundary);
        let mut stack = vec![(boundary, 0.0, -1)];
        while let Some((node,pos,dir)) = stack.pop() {
            if km0.get(&node).is_some() { continue; }
            km0.insert(node,pos);
            stack.push((inf.nodes[node].other_node, pos, -dir));
            uf.union(node, inf.nodes[node].other_node);
            if matches!(inf.nodes[node].edges, Edges::ModelBoundary) { boundaries.remove(&node); }
            match inf.nodes[node].edges {
                Edges::Single(b,d) => {
                    stack.push((b, pos + (dir as f64)*d, -dir));
                    //println!("edge from {} to {} dir {}", node, b, dir);
                    if !km0.contains_key(&b) { // TODO this smells a little
                        edges.push((node,b,(dir as f64)*d));
                    }
                },
                Edges::Switchable(obj) => {
                    if let StaticObject::Switch { right_link, left_link, .. } = inf.objects[obj] {
                        stack.push((right_link.0, pos + (dir as f64)*right_link.1, -dir));
                        stack.push((left_link.0, pos + (dir as f64)*left_link.1, -dir));
                        uf.union(node, right_link.0); // TODO holds only if lengths is zero here.
                        uf.union(node, left_link.0);  // but that is the case for the conversion
                                                      // as it is currently written in dgraph.rs.
                    } else { panic!(); }
                },
                _ => {},
            }
        }
    }


    // Now we have a first guess of the km, 
    // a list of edges,
    // and a map from original NodeIds to unknown variables

    //println!("edges {:?}", edges);

    let mut varmap : HashMap<NodeId, usize> = HashMap::new();
    let mut varidx = -1isize;
    let mut fixed :HashMap<NodeId, Option<usize>> = fixed.into_iter().map(|n| (n,None)).collect();
    for (n,km) in km0.iter() {
        let v = varmap.entry(uf.find_mut(*n)).or_insert_with(|| { varidx += 1; varidx as usize });
        fixed.entry(*n).and_modify(|nvar| { *nvar = Some(*v); });
    }

    let mut rhs : Vec<f64> = edges.iter().map(|(_,_,d)| *d).collect();
    for (_,km) in fixed.iter() { rhs.push(0.0); } // TODO fixed positions can be != 0.0 as well


    //println!("varmap {:?}", varmap);
    //println!("fixed {:?}", fixed);
    //println!("rhs {:?}", rhs);
    //for (n,var) in &varmap {
        //println!("var {:03}/{:03} km {:.3}", n, var, km0[n]);
    //}

     let params = lsqr::Params {
         damp: 0.0,
         rel_mat_err: 1e-6,
         rel_rhs_err: 1e-6,
         condlim :0.0,
         iterlim: inf.nodes.len(), 
     };

    // Now we have a first guess of the km, indexed by actual variables to solve
     let (sol,stats) = lsqr::lsqr(|msg| {}, //println!("{}", msg),
                edges.len() + fixed.len(), varmap.len(), 
                params,
                |prod| {
                    match prod {
                        lsqr::Product::YAddAx { x, y } => {
                            //println!("YAddAx");
                            //println!("x = {:?}", x);
                            //println!("y = {:?}", y);
                            // compute y += A * x
                            // y contains edge adjustments(?), x contains node kms
                            // [n_e +1, n_n] * [n_n, 1] = [n_e +1, 1]
                           for (e, (a,b,d)) in edges.iter().enumerate() {
                               let (a,b) = (varmap[&uf.find_mut(*a)], varmap[&uf.find_mut(*b)]);
                               y[e] += x[b] - x[a];
                           }
                           for (i,(_,v)) in fixed.iter().enumerate() { if let Some(var) = v { 
                               //println!("edges.len()  = {}", edges.len());
                               //println!("i = {}", i);
                               //println!("var = {}", *var);
                               y[edges.len() + i] += x[*var]; 
                           } }
                             //println!("YAddAx end");
                            //println!("x = {:?}", x);
                            //println!("y = {:?}", y);
                        },
                        lsqr::Product::XAddATy { x, y } => {
                            //println!("XAddATy");
                            //println!("x = {:?}", x);
                            //println!("y = {:?}", y);
                            // compute x += A^T * y
                            // y contains something, 
                            for (e, (a,b,d)) in edges.iter().enumerate() {
                                let (a,b) = (varmap[&uf.find_mut(*a)], varmap[&uf.find_mut(*b)]);
                                x[a] += -1. * y[e];
                                x[b] +=  1. * y[e];
                            }
                           for (i,(_,v)) in fixed.iter().enumerate() { if let Some(var) = v { 
                                x[*var] += y[edges.len() + i] ;
                           } }
                            //println!("XAddATy end");
                            //println!("x = {:?}", x);
                            //println!("y = {:?}", y);
                        },
                    }
                },
                &mut rhs);

     //println!("sol {:?}", sol);

     // This should be our solution, now we can map back to node kms
     let km :HashMap<NodeId,f64> = km0.into_iter().map(|(n,_)| (n, sol[varmap[&uf.find_mut(n)]])).collect();

     let mut v = km.iter().collect::<Vec<_>>();
     v.sort_by_key(|a| a.0);
     for (n,k) in v {
         if n%2 == 0 {
             //println!("node {:03} km {:.3}", n, k);
         }
     }

     //println!("solver stats {:#?}", stats);

     km
}



