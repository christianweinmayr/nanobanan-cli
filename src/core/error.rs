use thiserror::Error;

#[derive(Error, Debug)]
pub enum BananaError {
    #[error("API key not configured. Set GEMINI_API_KEY environment variable or run: banana config set api.key <your-key>")]
    MissingApiKey,

    #[error("API error: {message}")]
    ApiError {
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Request timeout")]
    Timeout,
}

impl From<reqwest::Error> for BananaError {
    fn from(err: reqwest::Error) -> Self {
        BananaError::ApiError {
            message: err.to_string(),
            source: Some(err),
        }
    }
}

impl From<rusqlite::Error> for BananaError {
    fn from(err: rusqlite::Error) -> Self {
        BananaError::DatabaseError(err.to_string())
    }
}
