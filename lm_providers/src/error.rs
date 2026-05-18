use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Context overflow: {0}")]
    ContextOverflow(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Tool validation failed: {0}")]
    ToolValidationError(String),

    #[error("Stream ended unexpectedly")]
    StreamEnded,

    #[error("Request aborted")]
    Aborted,

    #[error("Provider error: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider already exists: {0}")]
    ProviderExists(String),

    #[error("Registry lock poisoned")]
    LockPoisoned,
}
