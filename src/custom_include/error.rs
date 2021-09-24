#[derive(Debug)]
pub enum CustomIncludeError {
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    SelectorError(String)
}

impl From<std::io::Error> for CustomIncludeError {
    fn from(err: std::io::Error) -> Self {
        CustomIncludeError::IOError(err)
    }
}

impl From<serde_json::Error> for CustomIncludeError {
    fn from(err: serde_json::Error) -> Self {
        CustomIncludeError::JsonError(err)
    }
}