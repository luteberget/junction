#![allow(unused_imports)]
use rolling::input::staticinfrastructure as rolling_inf;
use std::collections::{HashMap, HashSet};
use ordered_float::OrderedFloat;
use std::sync::Arc;
use crate::model::*;
use crate::objects::*;
use crate::topology::*;
use matches::matches;

pub type ModelNodeId = Pt;
pub type ModelObjectId = PtA;

#[derive(Debug)]
pub struct DGraph {
    pub rolling_inf :rolling_inf::StaticInfrastructure, 
    pub node_ids :HashMap<rolling_inf::NodeId, Pt>,
    pub object_ids :HashMap<rolling_inf::ObjectId, PtA>,
    pub tvd_sections :HashMap<rolling_inf::ObjectId, 
        Vec<(rolling_inf::NodeId, rolling_inf::NodeId)>>,
    pub edge_lines :HashMap<(rolling_inf::NodeId, rolling_inf::NodeId), Vec<PtC>>,
}

pub fn edge_length(rolling_inf :&rolling_inf::StaticInfrastructure, a :rolling_inf::NodeId, b: rolling_inf::NodeId) -> Option<f64> {
    match rolling_inf.nodes[a].edges {
        rolling_inf::Edges::Single(bx,d) if b == bx => Some(d),
        rolling_inf::Edges::Switchable(objid) => {
            if let rolling_inf::StaticObject::Switch { left_link, right_link, .. } = rolling_inf.objects[objid] {
                if left_link.0 == b { Some(left_link.1) }
                else if right_link.0 == b { Some(right_link.1) }
                else { None }
            } else { None }
        }
        _ => None,
    }
}

pub struct DGraphBuilder {
    dgraph :rolling_inf::StaticInfrastructure,
    edge_tracks :HashMap<(rolling_inf::NodeId, rolling_inf::NodeId), Interval>,
}

#[derive(Debug)]
pub struct Interval {
    track_idx: usize,
    start: f64,
    end :f64,
}

impl DGraphBuilder {
    pub fn convert(model :&Model, topology :&Topology) -> Result<DGraph, ()> {
        let mut m = DGraphBuilder::new();

        let tracks = &topology.tracks;
        let locs = &topology.locations;
        let trackobjects = &topology.trackobjects;

        // Create signals objects separately (they are not actually part of the "geographical" 
        // infrastructure network, they are merely pieces of state referenced by sight objects)
        let mut static_signals :HashMap<PtA, rolling_inf::ObjectId> = HashMap::new();
        for (p,o) in model.objects.iter().filter(|(p,o)| matches!(o.symbol.shape, Shape::Signal))  {
            let id = m.new_object(rolling_inf::StaticObject::Signal);
            static_signals.insert(*p,id);
        }

        let mut signal_cursors : HashMap<PtA, Cursor> = HashMap::new();
        let mut detector_nodes : HashSet<(rolling_inf::NodeId, rolling_inf::NodeId)> = HashSet::new();
        let mut object_ids = HashMap::new();
        let node_ids = m.create_network(
            tracks, &locs, 
            |track_idx,mut cursor,dg| {
                let mut last_pos = 0.0;
                let mut objs :Vec<(f64,PtA,Function,Option<AB>)> = trackobjects[&track_idx].clone();
                objs.sort_by_key(|(pos,_,_,_)| OrderedFloat(*pos));
                for (pos, id, func, dir) in objs {
                    cursor = cursor.advance_single(&dg.dgraph, pos - last_pos).unwrap();
                    cursor = dg.insert_node_pair(cursor);

                    match func {
                        Function::Detector => { detector_nodes.insert(cursor.nodes(&dg.dgraph)); },
                        Function::MainSignal => { 
                            let c = if matches!(dir,Some(AB::B)) { cursor.reverse(&dg.dgraph) } else { cursor };
                            signal_cursors.insert(id,c); 
                            let (_cursor, obj) = dg.insert_object(cursor, 
                                                                  rolling_inf::StaticObject::Signal);
                            object_ids.insert(obj, id);
                        },
                    }
                    last_pos = pos;
                }
            } );

        // Sight to signals
        for (id,cursor) in signal_cursors {
            let objid = static_signals[&id];
            let sight_dist = 200.0; // TODO configurable
            for (cursor,dist) in cursor.reverse(&m.dgraph).advance_nontrailing_truncate(&m.dgraph, sight_dist) {
                m.insert_object(cursor, rolling_inf::StaticObject::Sight{
                    distance: dist, signal: objid,
                });
            }
        }

        // Train detectors
        for (node_idx,node) in m.dgraph.nodes.iter().enumerate() {
            if matches!(node.edges, rolling_inf::Edges::ModelBoundary) {
                detector_nodes.insert((node_idx, node.other_node));
            }
        }
        let tvd_sections = route_finder::detectors_to_sections(&mut m.dgraph, &detector_nodes)
            .expect("could not calc tvd sections.");

        let mut edge_lines :HashMap<(rolling_inf::NodeId, rolling_inf::NodeId), Vec<PtC>>
            = m.edge_tracks.into_iter()
            .map(|(edge,Interval { track_idx, start, end })| 
                 (edge, topology.interval_map(track_idx,start,end))).collect();

        let rev_edge_lines = edge_lines.iter().map(|((a,b),v)| ((*b,*a),v.clone())).collect::<Vec<_>>();
        edge_lines.extend(rev_edge_lines.into_iter());


        Ok(DGraph {
            rolling_inf: m.dgraph,
            node_ids: node_ids,
            object_ids: object_ids,
            tvd_sections: tvd_sections,
            edge_lines: edge_lines,
        })

    }

    pub fn new() -> DGraphBuilder {
        let model = rolling_inf::StaticInfrastructure {
            nodes: Vec::new(), 
            objects: Vec::new(),
        };
        DGraphBuilder { dgraph: model, edge_tracks: HashMap::new() }
    }

    pub fn new_object(&mut self, obj :rolling_inf::StaticObject) -> rolling_inf::ObjectId {
        let id  = self.dgraph.objects.len();
        self.dgraph.objects.push(obj);
        id
    }

    pub fn new_object_at(&mut self, obj :rolling_inf::StaticObject, node: rolling_inf::NodeId) -> rolling_inf::ObjectId {
        let obj_id = self.new_object(obj);
        self.dgraph.nodes[node].objects.push(obj_id);
        obj_id
    }

    pub fn new_node_pair(&mut self) -> (rolling_inf::NodeId, rolling_inf::NodeId) {
        let (i1,i2) = (self.dgraph.nodes.len(), self.dgraph.nodes.len() +1);
        self.dgraph.nodes.push(rolling_inf::Node { other_node: i2,
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        self.dgraph.nodes.push(rolling_inf::Node { other_node: i1,
            edges: rolling_inf::Edges::Nothing, objects: Default::default() });
        (i1,i2)
    }

    fn connect_linear(&mut self, na :rolling_inf::NodeId, nb :rolling_inf::NodeId, d :f64) {
        self.dgraph.nodes[na].edges = rolling_inf::Edges::Single(nb, d);
        self.dgraph.nodes[nb].edges = rolling_inf::Edges::Single(na, d);
    }

    fn split_edge(&mut self, a :rolling_inf::NodeId, b :rolling_inf::NodeId, second_dist :f64) -> (rolling_inf::NodeId, rolling_inf::NodeId) {
        let (na,nb) = self.new_node_pair();
        let reverse_dist = edge_length(&self.dgraph, b, a).unwrap();
        let forward_dist = edge_length(&self.dgraph, a, b).unwrap();
        let first_dist = reverse_dist - second_dist;
        self.replace_conn(a,b,na,first_dist);
        self.replace_conn(b,a,nb,second_dist);

        // TODO this seems overcomplicated
        for ((a1,a2),(b1,b2)) in vec![((a,na),(b,nb)),((b,nb),(a,na))].into_iter() {
            if let Some(Interval { track_idx, start, end }) = self.edge_tracks.remove(&(a1,b1)) {
                self.edge_tracks.insert((a1,a2), Interval { track_idx, 
                    start: start, end: start+first_dist });
                self.edge_tracks.insert((b2,b1), Interval { track_idx, 
                    start: start+first_dist, end: end });
            }
        }
        (na,nb)
    }


    fn replace_conn(&mut self, a :rolling_inf::NodeId, b :rolling_inf::NodeId, x :rolling_inf::NodeId, d :f64) {
        use rolling_inf::Edges;
        match self.dgraph.nodes[a].edges {
            Edges::Single(bx,_dist) if b == bx => { self.dgraph.nodes[a].edges = Edges::Single(x,d); }
            Edges::Switchable(objid) => {
                if let rolling_inf::StaticObject::Switch { ref mut left_link, ref mut right_link, .. } = &mut self.dgraph.objects[objid] {
                    if left_link.0 == b { *left_link = (x,d); }
                    else if right_link.0 == b { *right_link = (x,d); }
                    else { panic!() }
                } else { panic!() }
            }
            _ => { panic!() },
        };
        self.dgraph.nodes[x].edges = Edges::Single(a,d);
    }

    pub fn insert_node_pair(&mut self, at :Cursor) -> Cursor {
        match at {
            Cursor::Node(x) => Cursor::Node(x),
            Cursor::Edge((a,b),d) => {
                let (na,nb) = self.split_edge(a,b,d);
                Cursor::Node(nb)
            },
        }
    }

    pub fn insert_object(&mut self, at :Cursor, obj :rolling_inf::StaticObject) -> (Cursor,rolling_inf::ObjectId) {
        if let Cursor::Node(a) = at {
            let objid = self.new_object_at(obj, a);
            (at,objid)
        } else {
            let at = self.insert_node_pair(at);
            self.insert_object(at, obj)
        }
    }

    pub fn create_network(&mut self,
        tracks: &[(f64, (Pt, Port), (Pt, Port))], // track length and line pieces
        nodes: &HashMap<Pt,(NDType, Vc)>,
        mut each_track: impl FnMut(usize,Cursor,&mut Self)) -> HashMap<rolling_inf::NodeId, Pt> {

        let mut node_ids = HashMap::new();
        let mut ports :HashMap<(Pt,Port), rolling_inf::NodeId>  = HashMap::new();
        for (i,(len,a,b)) in tracks.iter().enumerate() {
            let (start_a,start_b) = self.new_node_pair();
            let (end_a,end_b) = self.new_node_pair();
            ports.insert(*a, start_a);
            self.connect_linear(start_b, end_a, *len);
            ports.insert(*b, end_b);
            self.edge_tracks.insert((start_b,end_a), Interval { track_idx: i, 
                start: 0.0, end: *len });
            each_track(i,Cursor::Node(start_b), self);
        }

        for (pt,(node,_)) in nodes.iter() {
            match node {
                NDType::BufferStop => {},
                NDType::OpenEnd => {
                    self.dgraph.nodes[ports[&(*pt, Port::End)]].edges =
                        rolling_inf::Edges::ModelBoundary;
                    node_ids.insert(ports[&(*pt,Port::End)], *pt);
                },
                NDType::Cont => {
                    self.connect_linear(ports[&(*pt, Port::ContA)],
                                        ports[&(*pt, Port::ContB)], 0.0);
                },
                NDType::Sw(side) => {
                    let sw_obj = self.new_object(rolling_inf::StaticObject::Switch {
                        left_link:  (ports[&(*pt,Port::Left)], 0.0),
                        right_link: (ports[&(*pt,Port::Right)], 0.0),
                        branch_side: *side,
                    });

                    self.dgraph.nodes[ports[&(*pt, Port::Left)]].edges  = 
                        rolling_inf::Edges::Single(ports[&(*pt,Port::Trunk)], 0.0);
                    self.dgraph.nodes[ports[&(*pt, Port::Right)]].edges = 
                        rolling_inf::Edges::Single(ports[&(*pt,Port::Trunk)], 0.0);
                    self.dgraph.nodes[ports[&(*pt, Port::Trunk)]].edges =
                        rolling_inf::Edges::Switchable(sw_obj);
                },
                _ => unimplemented!(),
            }
        }
        node_ids
    }
}

#[derive(Copy,Clone, Debug)]
pub enum Cursor {
    Node(rolling_inf::NodeId),
    Edge((rolling_inf::NodeId, rolling_inf::NodeId), f64), // remaining distance along edge
}

fn edge_multiplicity(e :&rolling_inf::Edges) -> usize {
    match e {
        rolling_inf::Edges::Switchable(_) => 2,
        rolling_inf::Edges::ModelBoundary |
        rolling_inf::Edges::Nothing => 0,
        rolling_inf::Edges::Single(_,_) => 1,
    }
}

fn out_edges(dg :&rolling_inf::StaticInfrastructure, e: &rolling_inf::NodeId) -> Vec<(rolling_inf::NodeId,f64)> {
    match dg.nodes[*e].edges {
        rolling_inf::Edges::Single(n,d) => vec![(n,d)],
        rolling_inf::Edges::Switchable(obj) => match dg.objects[obj] {
            rolling_inf::StaticObject::Switch { right_link, left_link, .. } => vec![right_link,left_link],
            _ => panic!(),
        },
        rolling_inf::Edges::ModelBoundary | rolling_inf::Edges::Nothing => vec![],
    }
}

impl Cursor {
    pub fn advance_single(&self, dg :&rolling_inf::StaticInfrastructure, l :f64) -> Option<Cursor> {
        if l <= 0.0 { return Some(*self); }
        match self {
            Cursor::Node(n) => {
                match dg.nodes[*n].edges {
                    rolling_inf::Edges::Single(b,d) => Cursor::Edge((*n,b),d).advance_single(dg, l),
                    _ => None,
                }
            }
            Cursor::Edge((a,b),d) => if *d > l {
                Some(Cursor::Edge((*a,*b), *d - l))
            } else {
                Cursor::Node(*b).advance_single(dg, l - *d)
            },
        }
    }

    pub fn advance_nontrailing_truncate(&self, dg :&rolling_inf::StaticInfrastructure, l :f64) -> Vec<(Cursor,f64)> {
        let mut output = Vec::new();
        let mut cursors = vec![(*self,l)];
        while let Some((cursor,d)) = cursors.pop() {
            match cursor {
                Cursor::Edge((a0,b0),nd0) => {
                    if nd0 >= d { 
                        output.push((Cursor::Edge((a0,b0),nd0-d), l)); // Done: Full length achieved
                    } else {
                        if edge_multiplicity(&dg.nodes[b0].edges) > 1 {
                            output.push((Cursor::Edge((a0,b0),0.0), l - (d - nd0))); // Done: Trailing switch, truncate path here
                        } else {
                            let a = dg.nodes[b0].other_node;
                            for (b,nd) in out_edges(dg,&a)  {
                                cursors.push((Cursor::Edge((a,b),nd), d - nd0));
                            }
                        }
                    }
                },
                Cursor::Node(a) => {
                    for (b,nd) in out_edges(dg, &a) {
                        cursors.push((Cursor::Edge((a,b),nd), d));
                    }
                },
            };
        }
        output
    }

    pub fn nodes(&self, dg :&rolling_inf::StaticInfrastructure) -> (rolling_inf::NodeId, rolling_inf::NodeId) {
        match self {
            Cursor::Node(n) => (*n, dg.nodes[*n].other_node),
            Cursor::Edge((a,b),_d) => (*a,*b),
        }
    }

    pub fn reverse(&self, dg :&rolling_inf::StaticInfrastructure) -> Cursor {
        match self {
            Cursor::Node(n) => Cursor::Node(dg.nodes[*n].other_node),
            _ => unimplemented!(),
        }
    }

}
