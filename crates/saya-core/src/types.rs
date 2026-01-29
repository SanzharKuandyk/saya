use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum AppEvent {
    ConfigChanged,
    UiEvent(UiEvent),
}

#[derive(Serialize, Deserialize)]
pub enum UiEvent {}
