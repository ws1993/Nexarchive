use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("llm response invalid: {0}")]
    InvalidLlmResponse(String),
    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
