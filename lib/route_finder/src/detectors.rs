use petgraph;
use rolling::input::staticinfrastructure::*;
use std::collections::{HashSet, HashMap};

pub fn detectors_to_sections(m :&mut StaticInfrastructure, detector_nodes :&HashSet<(NodeId,NodeId)>) 
    -> Result<HashMap<ObjectId, Vec<(NodeId,NodeId)>>, String> {
    let is_boundary_idx = 0;
    let mut sets = petgraph::unionfind::UnionFind::new(m.nodes.len() + 1);

    for (i,n) in m.nodes.iter().enumerate() {

        let is_detector = 
            detector_nodes.contains(&(i,n.other_node)) ||
            detector_nodes.contains(&(n.other_node,i));

        // Join node pairs which do not contain a detector
        if !is_detector {
            sets.union(i +1, n.other_node +1);
        }
        
        // Join edges
        match n.edges {
            Edges::Single(other,_) => { sets.union(i + 1, other + 1); },
            Edges::Switchable(id) => {
                if let StaticObject::Switch { left_link, right_link, .. } = &m.objects[id] {
                    sets.union(i  +1, left_link.0  +1);
                    sets.union(i  +1, right_link.0  +1);
                } else {
                    return Err(format!("DGraph is inconsistent."));
                }
            },
            Edges::ModelBoundary => { sets.union(i + 1, is_boundary_idx); },
            Edges::Nothing => {},
        }
    }

    let mut sec_num_map = HashMap::new();

    // Go back to each node and insert tvd entry/exit

    fn new_object(objs :&mut Vec<StaticObject>, obj :StaticObject) -> ObjectId {
        let id = objs.len();
        objs.push(obj);
        id
    }

    let mut inserts = Vec::new();
    for (i,n) in m.nodes.iter().enumerate() {
        let is_detector = 
            detector_nodes.contains(&(i,n.other_node)) ||
            detector_nodes.contains(&(n.other_node,i));
        if is_detector {
            let section = sets.find(i + 1);
            if section != sets.find(is_boundary_idx) {
                if sec_num_map.get(&section).is_none() {
                    // create new section object in staticinfrastructure
                    let objid = new_object(&mut m.objects, StaticObject::TVDSection);
                    sec_num_map.insert(section, objid);
                }

                // make new entry/exit objects
                let tvd = sec_num_map[&section];
                let entry = new_object(&mut m.objects, StaticObject::TVDLimit { enter: Some(tvd), exit:  None });
                let exit  = new_object(&mut m.objects, StaticObject::TVDLimit { exit:  Some(tvd), enter: None });

                // Attach these objects to their nodes
                inserts.push((i, exit));
                inserts.push((n.other_node, entry));
            } else {
                // Section connects to a boundary, we don't make it a detection section.
            }
        }
    }

    for (i,o) in inserts {
        m.nodes[i].objects.push(o);
    }

    let mut sec_edges = HashMap::new();

    println!("sec num map {:?}",sec_num_map);
    for (i,n) in m.nodes.iter().enumerate() {
        let section = sets.find(i +1);
        match n.edges {
            Edges::Single(other_id, _l) => {
                if sec_num_map.contains_key(&section) && 
                    section != sets.find(is_boundary_idx) && 
                    section == sets.find(other_id +1) {
                    sec_edges.entry(sec_num_map[&section]).or_insert(Vec::new())
                        .push((i,other_id));
                }
            },
            _ => {} // Other edges are zero length in the conversion for now...  TODO.
        }
    }

    Ok(sec_edges)


}
