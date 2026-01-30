use serde::{Deserialize, Serialize};

/// TODO: Define proper purpose for NetworkConfig:
/// or be combined(don't like this)
#[derive(Default, Serialize, Deserialize)]
pub struct NetworkConfig {}

impl NetworkConfig {
    pub fn new() -> Self {
        Self {}
    }
}
