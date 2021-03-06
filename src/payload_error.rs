use crate::{selector_error::SelectorError};
use crate::custom_include::error::CustomIncludeError;

#[derive(Debug)]
pub enum PayloadError {
    IOError(std::io::Error),
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

impl From<CustomIncludeError> for PayloadError {
    fn from(err: CustomIncludeError) -> Self {
        match err {
            CustomIncludeError::IOError(e) => PayloadError::IOError(e),
            CustomIncludeError::JsonError(e) => PayloadError::JsonError(e),
            CustomIncludeError::SelectorError(e) => PayloadError::SelectorError(SelectorError::new(&e, None)),
        }
    }
}