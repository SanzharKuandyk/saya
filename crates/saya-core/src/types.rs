#[derive(Debug)]
pub enum AppEvent {
    ConfigChanged,
    UiEvent(UiEvent),
    ApiRequest(ApiRequest),
}

#[derive(Debug)]
pub enum UiEvent {}

#[derive(Debug)]
pub enum ApiRequest {}
