use crate::document::topology::*;
use crate::document::model::*;
use crate::document::objects::*;
use crate::synthesis::*;

/// An initial guess for signal and detector placement.
/// Places a signal at 0 and 150 m from every switch.
pub fn initial_design(topo :&Topology) -> Design {

    let stock_length = 23.0;
    let fouling_length = 51.0;
    let overlap_lengths = vec![0.0, 150.0];

    let mut objects = Vec::new();

    for (track_idx,(length,(_,port_a),(_,port_b))) in topo.tracks.iter().enumerate()  {
        for (pos, port,dir) in &[(0.0, port_a, AB::A),(*length, port_b, AB::B)] {
            match port {
                Port::Trunk => { // set a detector at the stock
                    for c in cur_move(topo, Cursor { tr: track_idx, pos: *pos, dir: *dir }, stock_length) {
                        objects.push((c.tr, c.pos, Function::Detector, None));
                    }
                }
                Port::Left | Port::Right => { // set a signal and detector at each overlap length
                    for overlap_length in &overlap_lengths {
                        let l = fouling_length + overlap_length;
                        for c in cur_move(topo, Cursor { tr: track_idx, pos: *pos, dir: *dir}, l) {
                            objects.push((c.tr,c.pos,Function::Detector,None));
                            objects.push((c.tr,c.pos,Function::MainSignal { has_distant: true },Some(c.dir.other())));
                        }
                    }
                },

                _ => {}, // TODO crossings
            }
        }
    }

    objects
}

struct Cursor {
    tr :usize,
    pos :f64,
    dir :AB,
}

struct IntervalPos(pub (f64,f64), pub f64);
impl IntervalPos {
    pub fn add(&self, y :f64) -> Result<IntervalPos, f64> {
        let &IntervalPos((a,b),x) = self;
        let z = x + y;
        if a <= z && z <= b {
            Ok(IntervalPos((a,b), z))
        } else {
            Err((a-z).abs().min((b-z).abs()))
        }
    }
}

fn cur_move(topo :&Topology, c :Cursor, length: f64) -> Vec<Cursor> {
    let track_length = topo.tracks[c.tr].0;
    match IntervalPos((0.0, track_length), c.pos).add(c.dir.factor()*length) {
        Ok(IntervalPos(_,x)) => vec![Cursor { tr: c.tr, pos: x, dir: c.dir }],
        Err(remaining) => other_cursors(topo, c.tr, c.dir).into_iter()
            .flat_map(|c| cur_move(topo, c, remaining)).collect(),
    }
}

fn other_cursors(topo :&Topology, tr :usize, dir :AB) -> Vec<Cursor> {
    let mut output = Vec::new();
    let (pt,port) = match dir {
        AB::A => &topo.tracks[tr].2,
        AB::B => &topo.tracks[tr].1,
    };
    for (i,(l,(pt_a,port_a),(pt_b,port_b))) in topo.tracks.iter().enumerate() {
        if pt_a == pt && port.is_opposite(port_a) {
            output.push(Cursor { tr: i, pos: 0.0, dir: AB::A });
        }
        if pt_b == pt && port.is_opposite(port_b) {
            output.push(Cursor { tr: i, pos: *l, dir: AB::B });
        }
    }
    output
}

