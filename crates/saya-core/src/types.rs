#[derive(Debug, Clone)]
pub enum AppEvent {
    ConfigChanged,
    UiEvent(UiEvent),
    ApiRequest(ApiRequest),
    TextInput(String),
    RawTextInput {
        text: String,
        source: TextSource,
    },
    ShowResults(Vec<DisplayResult>),
    CreateCard(DisplayResult),
    TriggerOcr {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    CaptureWindow {
        window_id: Option<u32>,
    },
    OcrStatusUpdate {
        status: String,
        capturing: bool,
    },
    BackendReady,
}

#[derive(Debug, Clone)]
pub enum TextSource {
    Ocr,
    Clipboard,
    Websocket,
    Manual,
}

#[derive(Debug, Clone)]
pub struct DisplayResult {
    pub term: String,
    pub reading: String,
    pub definition: String,
    pub frequency: Option<String>,
    pub pitch_accent: Option<String>,
    pub jlpt_level: Option<String>,
    pub conjugation: Option<String>,
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
