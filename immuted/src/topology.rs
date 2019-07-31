use std::collections::{BTreeSet, BTreeMap,HashMap};
use crate::model::*;

pub type Tracks = Vec<(f64,(usize,Port),(usize,Port))>;
pub type Locations = Vec<(Pt,NDType,Vc)>;

fn to_vec(pt :(i32,i32)) -> Pt { nalgebra_glm::vec2(pt.0,pt.1) }

pub fn convert<'a>(pieces :impl IntoIterator<Item = &'a (Pt,Pt)>,
                  node_overrides :&im::HashMap<Pt,NDType>,
                  def_len :f64) -> Result<(Tracks,Locations,im::HashMap<Pt,NDType>), ()>{

    #[derive(Debug, Copy, Clone)]
    enum AB { A, B }

    let mut tracks :Vec<(Pt,Pt,f64)> = Vec::new();
    let mut locs :HashMap<(i32,i32), Vec<((usize,AB),Pt)>> = HashMap::new();

    let mut p = SymSet::new();
    for (a,b) in pieces.into_iter() {
        p.insert(((a.x,a.y),(b.x,b.y)));
    }
    let mut pieces = p;

    while let Some((p1,p2)) = pieces.remove_any() {
        let mut length = def_len;
        let (mut a, mut b) = ((p1,p2),(p2,p1));
        drop(p1);drop(p2);

        let mut extend = |p :&mut ((i32,i32),(i32,i32)), other: (i32,i32)| {
            loop {
                if locs.contains_key(&p.0) || p.0 == other { break; }
                if let Some(n) = pieces.remove_single(p.0) {
                    *p = (n,p.0);
                    length += def_len;
                } else {
                    break;
                }
            }
        };

        extend(&mut a, b.0);
        extend(&mut b, a.0);
        let track_idx = tracks.len();
        tracks.push((to_vec(a.0),to_vec(b.0),length));
        locs.entry(a.0).or_insert(Vec::new()).push(((track_idx, AB::A), to_vec(a.1)));
        locs.entry(b.0).or_insert(Vec::new()).push(((track_idx, AB::B), to_vec(b.1)));
    }

    let mut tp : Vec<(Option<(usize,Port)>, Option<(usize,Port)>, f64)> =
        tracks.into_iter().map(|(_,_,l)| (None,None,l)).collect();

    let mut settr = |(i,ab) :(usize,AB), val| match ab {
        AB::A => tp[i].0 = val,
        AB::B => tp[i].1 = val,
    };

    let mut locx :HashMap<Pt,(NDType,Vc)> = HashMap::new();

    for (l_i, (p,conns)) in locs.into_iter().enumerate() {
        match conns.as_slice() {
            [(t,q)] => {
                settr(*t, Some((l_i, Port::End)));
                locx.insert(to_vec(p), (NDType::OpenEnd, *q - to_vec(p)));
            },
            _ => unimplemented!(),
        };
    }


    Ok(
        (tp.into_iter().map(|(a,b,l)| (l, a.unwrap(), b.unwrap())).collect(),
         locx.into_iter().map(|(p,(n,v))| (p,n,v)).collect(),
         node_overrides.clone() /* TODO */)
    )
}


#[derive(Debug,Clone)]
pub struct SymSet<T:Ord+Copy> {
    pub map :BTreeMap<T, BTreeSet<T>>,
}

impl<T:Ord+Copy> SymSet<T> {
    pub fn new() -> Self { SymSet { map: BTreeMap::new() } }

    pub fn iter(&self, mut f :impl FnMut(&T,&T)) {
        for (a,set) in self.map.iter() {
            for b in set {
                if !( a > b) {
                    f(a,b);
                }
            }
        }
    }

    pub fn insert(&mut self, pt :(T,T)) -> bool {
        let r1 = self.map.entry(pt.0).or_insert(BTreeSet::new()).insert(pt.1);
        let r2 = self.map.entry(pt.1).or_insert(BTreeSet::new()).insert(pt.0);
        if r1 != r2 { panic!(); }
        r1
    }

    pub fn remove(&mut self, pt :(T,T)) -> bool {
        let r1 = self.remove_oneway((pt.0,pt.1));
        let r2 = self.remove_oneway((pt.1,pt.0));
        if r1 != r2 { panic!(); }
        if r1 && self.map[&pt.0].is_empty() { self.map.remove(&pt.0); }
        if r2 && self.map[&pt.1].is_empty() { self.map.remove(&pt.1); }
        r1
    }

    fn remove_oneway(&mut self, pt :(T,T)) -> bool {
        self.map.get_mut(&pt.0).map(|s| s.remove(&pt.1)).unwrap_or(false)
    }

    pub fn contains(&self, val :(T,T)) -> bool {
        self.map.get(&val.0).map(|v| v.contains(&val.1)) == Some(true)
    }

    pub fn get_any(&self) -> Option<(T,T)> {
        let (e1,set) = self.map.iter().nth(0)?;
        let e2 = set.iter().nth(0)?;
        Some((*e1,*e2))
    }

    pub fn remove_any(&mut self) -> Option<(T,T)> {
        let elem = self.get_any()?;
        self.remove(elem);
        Some(elem)
    }

    pub fn remove_single(&mut self, val :T) -> Option<T> {
        let set = self.map.get_mut(&val)?;
        let other = *set.iter().nth(0)?;
        if set.len() != 1 { return None; }
        self.remove((val,other));
        Some(other)
    }

    pub fn from_iter(x :impl IntoIterator<Item = (T,T)>) -> Self {
        let mut s = SymSet::new();
        for i in x.into_iter() { s.insert(i); }
        s
    }
}

