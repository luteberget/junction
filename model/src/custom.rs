/// Custom data sets
///

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Custom {
    /// File name of lua script to use as custom data set.
    Script(String), 
}

