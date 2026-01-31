#[derive(Debug, Clone)]
pub enum AppEvent {
    ConfigChanged,
    UiEvent(UiEvent),
    ApiRequest(ApiRequest),
    TextInput(String),
    ShowResults(Vec<DisplayResult>),
    CreateCard(DisplayResult),
}

#[derive(Debug, Clone)]
pub struct DisplayResult {
    pub term: String,
    pub reading: String,
    pub definition: String,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    SearchText(String),
    SelectResult(usize),
    Show,
    Hide,
    Close,
}

#[derive(Debug, Clone)]
pub enum ApiRequest {}
