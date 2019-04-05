use log::*;
use glrail::model::*;

fn main() {
    // trace output
env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();


   use std::fs::File;
   use std::path::Path;

   let json_path = Path::new(&"overtake_noinf_long.ron");
   let json_file = File::open(json_path).unwrap();
   let model : Model = ron::de::from_reader(json_file).unwrap();

   debug!(" Loaded model.");
   debug!("{:?}", model.inf);

   let usages = model.scenarios.iter().filter_map(|s| {
       if let glrail::scenario::Scenario::Usage(usage, _) = s { Some(usage.clone()) } else { None } })
       .collect::<Vec<_>>();
   glrail::analysis::synthesis::synthesis(&model.inf, &usages, &model.vehicles, |n,o| {
       debug!("Received model with score {}\n{:?}\n\n", n, o);
       return true;
   }).unwrap();
}
