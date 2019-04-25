/// Railway model for Junction. 
/// Contains:
///
///  * infrastructure (tracks, nodes, objects),
///  * interlocking (routes)
///  * dgraph 
///  * vehicle data
///  * scenarios (concrete dispatch and operational specs/movements)
///  * custom data set definitions (links to lua files?)
///  * options structs for interlocking derivation, schematic derivation, 
///    synthesis derivation, etc.
///

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub mod infrastructure;
pub mod interlocking;
pub mod scenario;
pub mod vehicle;
pub mod custom;

use infrastructure::*;
use scenario::*;
use vehicle::*;
use custom::*;

pub enum ModelUpdateResult {
    NoChange,
    InfrastructureChanged,
    InterlockingChanged,
    ScenarioChanged(usize),
}

pub enum ModelAction {
    Inf(InfrastructureEdit),
    Scenario(ScenarioEdit),
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    pub base_inf :Infrastructure,

    pub schematic_options: SchematicOptions,
    pub interlocking_options: InterlockingOptions,
    pub synthesis_options: SynthesisOptions,

    pub scenarios :Vec<Scenario>,
    pub vehicles :Vec<Vehicle>,
    pub custom_datasets: Vec<Custom>,
}

impl Model {
    pub fn empty() -> Model {
        Model {
            base_inf: Infrastructure::new_empty(),

            schematic_options: (),
            interlocking_options: (),
            synthesis_options: (),

            scenarios: Vec::new(),
            vehicles: Vec::new(),
            custom_datasets: Vec::new(),
        }
    }
}


pub type SchematicOptions = ();
pub type InterlockingOptions = ();
pub type SynthesisOptions = ();
