use crate::selector_error::SelectorError;

#[derive(Debug)]
pub enum PayloadError {
    JsonError(serde_json::Error),
    SelectorError(SelectorError)
}

impl From<serde_json::Error> for PayloadError {
    fn from(err: serde_json::Error) -> Self {
        PayloadError::JsonError(err)
    }
}

impl From<SelectorError> for PayloadError {
    fn from(err: SelectorError) -> Self {
        PayloadError::SelectorError(err)
    }
}