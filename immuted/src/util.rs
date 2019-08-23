use crate::model::{Pt,PtC};
use crate::ui::ImVec2;
use nalgebra_glm as glm;
use glm::I32Vec2;

pub fn order<T: Ord>(a :T, b: T) -> (T,T) {
    if b < a { (b,a) } else { (a,b) }
}

pub fn order_ivec(a :I32Vec2, b: I32Vec2) -> (I32Vec2,I32Vec2) {
    if a.x < b.x { (a,b) } else if a.x > b.x { (b,a) } else if a.y < b.y { (a,b) } else { (b,a) }
}

pub fn unit_step_diag_line(p1 :Pt, p2 :Pt) -> Vec<Pt> {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (0..=(dx.abs().max(dy.abs()))).map(move |d| glm::vec2(p1.x + d * dx.signum(), p1.y + d * dy.signum() ) ).collect()
}

pub fn route_line(from :Pt, to :Pt) -> Vec<(Pt,Pt)> {
	// diag
	let mut vec = Vec::new();
	let (dx,dy) = (to.x - from.x, to.y - from.y);
	let mut other = from;
	if dy.abs() > 0 {
		other = glm::vec2(from.x + dy.abs() * dx.signum(), from.y + dy );
		vec.push((from, other));
	}
	if dx.abs() > 0 {
		let other_dx = to.x - other.x;
		let goal = glm::vec2(other.x + if other_dx.signum() == dx.signum() { other_dx } else { 0 }, other.y );
		if other != goal {
			vec.push((other, goal));
		}
	}
	vec
}

pub fn point_in_rect(p :ImVec2, a :ImVec2, b :ImVec2) -> bool {
    let xl = a.x.min(b.x);
    let xh = a.x.max(b.x);
    let yl = a.y.min(b.y);
    let yh = a.y.max(b.y);
    xl <= p.x && p.x <= xh && yl <= p.y && p.y <= yh
}

pub fn project_to_line(p :PtC, a :PtC, b :PtC) -> (PtC,f32) {
    let t = glm::clamp_scalar(glm::dot(&(p-a),&(b-a)) / glm::distance2(&a,&b), 0.0, 1.0);
    (glm::lerp(&a,&b,t), t)
}

pub fn dist_to_line_sqr(p0 :PtC, a :PtC, b :PtC) -> (f32,f32) {
    let (p,param) = project_to_line(p0,a,b);
    (glm::length2(&(p - p0)), param)
}

pub fn to_imvec(p :PtC) -> ImVec2 {
    ImVec2 { x: p.x, y: -p.y }
}

pub fn to_vec(pt :(i32,i32)) -> Pt { nalgebra_glm::vec2(pt.0,pt.1) }

pub fn in_rect(pt :PtC, a :PtC, b :PtC) -> bool {
    let (x_lo,x_hi) = (a.x.min(b.x), a.x.max(b.x));
    let (y_lo,y_hi) = (a.y.min(b.y), a.y.max(b.y));
    (x_lo <= pt.x && pt.x <= x_hi && y_lo <= pt.y && pt.y <= y_hi)
}


pub trait VecMap<V> {
    fn vecmap_insert(&mut self, key :usize, value :V);
    fn vecmap_remove(&mut self, key :usize) -> bool;
    fn vecmap_get(&self, key :usize) -> Option<&V>;
}

impl<V> VecMap<V> for Vec<Option<V>> {
    fn vecmap_insert(&mut self, key :usize, value :V) {
        while self.len() < key+1 {
            self.push(None);
        }
        self[key] = Some(value);
    }

    fn vecmap_remove(&mut self, key :usize) -> bool {
        if let Some(slot) = self.get_mut(key) {
            if slot.is_some() {
                *slot = None;
                return true;
            } 
        } 
        false
    }

    fn vecmap_get(&self, key :usize) -> Option<&V> {
        if let Some(Some(e)) = self.get(key) {
            return Some(e);
        }
        None
    }
}

