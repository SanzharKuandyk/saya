use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AppEvent {
    ConfigChanged,
    ConfigUpdate {
        field: String,
        value: String,
    },
    UiEvent(UiEvent),
    ApiRequest(ApiRequest),
    TextInput(String),
    RawTextInput {
        text: String,
        source: TextSource,
    },
    ShowResults(Vec<DisplayResult>),
    CreateCard(DisplayResult),
    TriggerOcr(CaptureRegion),
    TriggerAutoOcr(CaptureRegion),
    UpdateCaptureRegion(CaptureRegion),
    CaptureWindow {
        window_id: Option<u32>,
    },
    OcrStatusUpdate {
        status: String,
        capturing: bool,
    },
    BackendReady,
    ShowTranslation {
        text: String,
        from_lang: String,
        to_lang: String,
    },
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
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
