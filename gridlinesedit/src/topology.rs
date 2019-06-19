use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use crate::pt::*;
use crate::symset::*;

// 
//
// Railway
//

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum NDType { OpenEnd, BufferStop, Cont, Sw(Side), Err }

#[derive(Debug,Copy,Clone)]
#[derive(Serialize,Deserialize)]
pub enum AB { A, B }

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum Port { End, ContA, ContB, Left, Right, Trunk, Err }

#[derive(Debug,Copy,Clone)]
#[derive(Serialize, Deserialize)]
pub enum Side { Left, Right }

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    pub fn to_port(&self) -> Port {
        match self {
            Side::Left => Port::Left,
            Side::Right => Port::Right,
        }
    }

    pub fn to_rotation(&self) -> i8 {
        match self {
            Side::Left => 1,
            Side::Right => -1,
        }
    }
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct Railway {
    pub locations: Vec<(Pt, NDType, Vc)>,
    pub tracks: Vec<((usize,Port),(usize,Port), f64)>,
}

#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct Objects { // TODO rename Railway->Topology and make Railway a container for topo and objs.

    // A=up/B=down here refers to the Railway.track orientation
    pub objects: Vec<(usize, f64, AB, Object)>, // track, offset, dir, objdata
}

#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct Object {
}

pub fn to_railway(mut pieces :SymSet<Pt>, 
                  node_overrides :&HashMap<Pt,NDType>, 
                  def_len :f64) -> Result<Railway, ()>{
    // while edges:
    // 1. pick any edge from bi-indexed set
    // 2. follow edge to nodes, removing nodes from set
    // 3. create track there, put ends into another set
    //let mut pieces = SymSet::from_iter(ls);
    let mut tracks :Vec<(Pt,Pt,f64)> = Vec::new();
    let mut locs :HashMap<Pt, Vec<((usize,AB),Pt)>> = HashMap::new();
    println!("PIECES {:?}", pieces);
    while let Some((p1,p2)) = pieces.remove_any() {
        println!("adding track starting in {:?}", (p1,p2));
        let mut length = def_len;
        let (mut a, mut b) = ((p1,p2),(p2,p1));
        drop(p1); drop(p2);
        let mut extend = |p :&mut (Pt,Pt), other :Pt| {
            loop {
                println!("Extending from {:?}", p.0);
                if locs.contains_key(&p.0) || p.0 == other  { break; /* Node exists. */ }
                if let Some(n) = pieces.remove_single(p.0) {
                    *p = (n,p.0);
                    length += def_len;
                } else {
                    break; // Either no more nodes along the path,
                           // or the path splits. In any case, add node here.
                }
            }
        };

        extend(&mut a,b.0); println!("Done extending {:?}", a); 
        extend(&mut b,a.0); println!("Done extending {:?}", b); 
        let track_idx = tracks.len();
        tracks.push((a.0,b.0,length));
        locs.entry(a.0).or_insert(Vec::new()).push(((track_idx, AB::A), a.1));
        locs.entry(b.0).or_insert(Vec::new()).push(((track_idx, AB::B), b.1));
        println!("after iter PIECES {:?}", pieces);
    }


        // Now we have tracks from node locations A/B
    // and locations with each track's incoming angles
    // We want to transform into 
    // 1. list of locations with node type and corresponding orientation,
    //      LIdx -> (Pt, NDType, Vc)
    // 2. Tracks with start/end links to locations and corresponding PORTS.
    //      TIdx -> ((LIdx,Port),(LIdx,Port),Length)

    println!("SO FAR SO GOOD");
    println!("{:?}", locs);
    println!("{:?}", tracks);

    let mut tp :Vec<(Option<(usize,Port)>,Option<(usize,Port)>,f64)> =
        tracks.into_iter().map(|(_,_,l)| (None,None,l)).collect();
    let mut settr = |(i,ab) : (usize,AB), val| match ab {
        AB::A => tp[i].0 = val,
        AB::B => tp[i].1 = val,
    };
    let mut locx :Vec<(Pt, NDType, Vc)> = Vec::new();
    for (l_i,(p,conns)) in locs.into_iter().enumerate() {
        match conns.as_slice() {
            [(t,q)] => {
                settr(*t,Some((l_i, Port::End)));
                locx.push((p, NDType::OpenEnd, pt_sub(*q,p)));
            },
            [(t1,q1),(t2,q2)] => {
                settr(*t1,Some((l_i,Port::ContA))); settr(*t2,Some((l_i,Port::ContB)));
                locx.push((p, NDType::Cont, pt_sub(*q1,p)));
            },
            [(t1,q1),(t2,q2),(t3,q3)] => {
                let track_idxs = [*t1,*t2,*t3];
                let qs = [*q1,*q2,*q3];
                let angle = [v_angle(pt_sub(*q1,p)), v_angle(pt_sub(*q2,p)), v_angle(pt_sub(*q3,p))];
                let permutations = &[[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]];
                let mut found = false;
                for pm in permutations {
                    let angle_diff = modu((angle[pm[2]]-angle[pm[1]]),8);
                    // p.0 is trunk, p.1 is straight, and p.2 is branch.
                    if !(angle[pm[0]] % 4 == angle[pm[1]] % 4 &&
                         angle_diff == 1 || angle_diff == 7) { 
                        continue; 
                    }
                    else { found = true; }

                    println!("SWITCH {} {} {}", 
                             angle[pm[0]], 
                             angle[pm[1]], 
                             angle[pm[2]]);
                    // TODO the side is not correct?
                    let side = if angle_diff == 1 { Side::Left } else { Side::Right };
                    settr(track_idxs[pm[0]],Some((l_i, Port::Trunk)));
                    settr(track_idxs[pm[1]],Some((l_i, side.opposite().to_port())));
                    settr(track_idxs[pm[2]],Some((l_i, side.to_port())));
                    locx.push((p, NDType::Sw(side), pt_sub(qs[pm[1]], p)));
                    break;
                }
                if !found { panic!("switch didn't work"); } // TODO add err values?
            },
            _ => unimplemented!(), // TODO
        };

        // Override node type
        if let Some(nd) = node_overrides.get(&p) {
            println!("OVERRIDE {:?}", locx.last());
            locx.last_mut().unwrap().1 = *nd;
            println!("OVERRIDE {:?}", locx.last());
        }
    }

    let tp :Vec<((usize,Port),(usize,Port),f64)> = tp.into_iter()
        .map(|(a,b,l)| (a.unwrap(),b.unwrap(),l)).collect();

        Ok(Railway {
        locations: locx,
        tracks: tp,
    })
}

