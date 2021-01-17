#[derive(Debug, thiserror::Error)]
/// An error representing all the errors that could happen
pub enum Error {
    #[error("Error Working with a TCP stream: {0}")]
    StreamError(#[from] std::io::Error),
    #[error("Error working with JSON: {0}")]
    JSONError(#[from] serde_json::Error),
    #[error("Invalid utf8 error: {0}")]
    UTF8Error(#[from] std::str::Utf8Error),
}
