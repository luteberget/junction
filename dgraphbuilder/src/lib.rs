use rolling::input::staticinfrastructure as rolling_inf;

#[derive(Copy,Clone)]
pub enum Cursor {
    Node(rolling_inf::NodeId),
    Edge((rolling_inf::NodeId, rolling_inf::NodeId), f64), // remaining distance along edge
}

impl Cursor {
    pub fn advance_single(&self, dg :&rolling_inf::StaticInfrastructure, l :f64) -> Option<Cursor> {
        if l <= 0.0 { return Ok(self); }
        match self {
            Cursor::Node(n) => match dg.nodes[n].edges {
                rolling_inf::Edges::Single(b,d) => Cursor::Edge((n,b),d).advance_single(l),
                _ => None,
            }
            Cursor::Edge((a,b),d) => if d > l {
                Some(Cursor::Edge((a,b), d - l))
            } else {
                Cursor::Node(b).advance_single(l - d)
            },
        }
    }

    // advance_single_truncate,
    // advance_multiple,
    // advance_multiple_truncate,
}

pub struct DGraphBuilder {
    dgraph :rolling_inf::StaticInfrastructure,
}

impl DGraphBuilder {
    fn connect_linear(&mut self, na :rolling_inf::NodeId, nb :rolling_inf::NodeId, d :f64) {
        self.dgraph.nodes[na] = rolling_inf::Edges::Single(nb, d);
        self.dgraph.nodes[nb] = rolling_inf::Edges::Single(na, d);
    }

    fn split_edge(&mut self, a :rolling_inf::NodeId, b :rolling_inf::NodeId, second_dist :f64) -> (rolling_inf::NodeId, rolling_inf::NodeId) {
        let (na,nb) = self.new_node_pair();
        let first_dist = match self.dgraph.nodes[b].edges {
            rolling_inf::Edges::Single(_,reverse) => second_dist - reverse,
            _ => panic!(), // TODO
        };
        self.connect_linear(a,na,first_dist);
        self.connect_linear(nb,b,second_dist);
        (na,nb)
    }

    pub fn insert_object(&mut self, at :Cursor, same_dir :bool, 
                         obj :rolling_inf::StaticObject) -> Cursor {
        match at {
            Cursor::Node(a) => {
                self.new_object_at(obj, if same_dir { a } else { self.dgraph.nodes[a].other_node });
            },
            Cursor::Edge((a,b),d) => {
                let (na,nb) = self.split_edge(a,b,d);
                self.insert_object(Cursor::Node(nb), same_dir, obj)
            }
        }
    }

    pub fn from_track_drawing(
        tracks: &[(f64, Port, Port)], // track length and line pieces
        nodes: &[NDType]) -> (DGraphBuilder, Vec<Cursor>) {

        let builder = DGraphBuilder { dgraph: rolling_inf::DGraph {
            nodes :Vec::new(),
            objects :Vec::new(),
        } };

        let ports = HashMap::new();
        let out_tracks : Vec<_> = tracks.iter().map(|(len,a,b)| {
            let (start_a,start_b) = builder.new_node_pair();
            let (end_a,end_b) = builder.new_node_pair();
            ports.insert(a, start_a);
            builder.connect_linear(start_b, end_a, len);
            ports.insert(b, end_b);
            Cursor::Node(start_b) // Return cursor at start of track
        }).colllect();

        for (i,node) in nodes.iter().enumerate() {
            match node {
                NDType::OpenEnd => {
                    builder.dgraph.nodes[ports[(i, Port::End)]].edges = 
                        rolling_inf::Edges::ModelBoundary;
                },
                NDType::Cont => {
                    builder.connect_linear(ports[(i, Port::ContA)],
                                           ports[(i, Port::ContB)], 0.0);
                },
                NDType::Switch(side) => {
                    let sw_obj = builder.new_object(rolling_inf::StaticObject::Switch {
                        left_link:  (ports[(i,Port::Left)], 0.0),
                        right_link: (ports[(i,Port::Right)], 0.0),
                        branch_side: side,
                    });

                    let back = rolling_inf::Edges::Single(ports[(i,Port::Trunk)], 0.0);
                    builder.dgraph.nodes[ports[(i, Port::Left)]]  = back;
                    builder.dgraph.nodes[ports[(i, Port::Right)]] = back;
                    builder.dgraph.nodes[ports[(i, Port::Trunk)]] = 
                        rolling_inf::Edges::Switchable(sw_obj);
                },
            }
        }

        // TODO Node IDs map
        // TODO Object IDs map

        println!("DGRAPH {:?}", builder.dgraph);


        (builder,out_tracks)
    }
}
