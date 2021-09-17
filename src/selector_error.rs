use crate::jq::JqError;

#[derive(Debug)]
pub struct SelectorError{
    pub message: String,
    cause: Option<JqError>
}

impl SelectorError {
    pub fn new(message: &str, cause: Option<JqError>) -> Self {
        Self {
            message: message.to_string(),
            cause: cause
        }
    }
}

impl std::error::Error for SelectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(cause) = self.cause.as_ref() {
            match cause {
                JqError::IoError(e) => Some(e),
                JqError::ParserError(e) => Some(e),
                JqError::NonScalarValueError(e) => Some(e),
            }
        }
        else {
            None
        }
    }

    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}