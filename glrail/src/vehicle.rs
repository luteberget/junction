use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    pub name :String,
    pub length :f32,
    pub max_accel :f32,
    pub max_brake :f32,
    pub max_velocity :f32,
}

pub fn default_vehicle() -> Vehicle {
    Vehicle {
        name: format!("New vehicle"),
        length: 110.0,
        max_accel: 1.05,
        max_brake: 0.95,
        max_velocity: 20.0,
    }
}
