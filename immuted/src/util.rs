use crate::model::Pt;
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
