mod model;
mod topo;
mod xml;

#[cfg(test)]
mod tests {
    use crate::xml;
    use crate::model;
    use crate::topo;
    #[test]
    fn it_works() {
        println!("Reading xml");
        let s = std::fs::read_to_string("railml_tutorial_tracks.railml").unwrap();
        let railml = xml::parse_railml(&s).expect("railml parse failed");
        println!(" Found railml {:#?}", railml);

        let topo = topo::convert_railml_topo(railml).expect("topo conversion failed");
        println!(" Found topology {:#?}", topo);
    }
}
