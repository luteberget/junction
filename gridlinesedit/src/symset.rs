use std::collections::{BTreeSet, BTreeMap};
use serde::{Deserialize, Serialize};

#[derive(Debug,Clone)]
#[derive(Serialize,Deserialize)]
pub struct SymSet<T:Ord+Copy> {
    pub map :BTreeMap<T, BTreeSet<T>>,
}

impl<T:Ord+Copy> SymSet<T> {
    pub fn new() -> Self { SymSet { map: BTreeMap::new() } }

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

