use serde::Serialize;

#[allow(non_snake_case)] // backwards compatibility
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum CribbageErrorKind {
    BadCard,
    ParseError,
    BadHand,
    BadCount,
}
///
/// a hopefully useful message that tells the caller why the Web API failed
#[derive(Debug, Serialize)]
pub struct CribbageError {
    pub error_kind: CribbageErrorKind,
    pub message: String,
}

impl CribbageError {
    pub fn new(kind: CribbageErrorKind, msg: String) -> CribbageError {
        CribbageError {
            error_kind: kind,
            message: msg,
        }
    }
}
