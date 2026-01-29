use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct UiConfig {}

impl UiConfig {
    pub fn new() -> Self {
        Self {}
    }
}
