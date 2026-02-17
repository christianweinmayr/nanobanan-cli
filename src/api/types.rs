use serde::{Deserialize, Serialize};

/// Request body for generateContent endpoint
#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
}

/// Content block (user or model message)
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(default)]
    pub parts: Vec<ContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Part of content (text or image)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(alias = "inline_data", alias = "inlineData")]
        inlineData: InlineData,
    },
}

/// Inline image data
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineData {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

/// Generation configuration
#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<ImageConfig>,
}

/// Image-specific configuration
#[derive(Debug, Serialize)]
pub struct ImageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
}

/// Safety settings
#[derive(Debug, Serialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

/// Response from generateContent endpoint
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_metadata: Option<UsageMetadata>,
}

/// A candidate response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Option<Content>,
    pub finish_reason: Option<String>,
    pub finish_message: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Feedback about the prompt
#[derive(Debug, Deserialize)]
pub struct PromptFeedback {
    pub block_reason: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Safety rating for content
#[derive(Debug, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Token usage metadata
#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    pub prompt_token_count: Option<i32>,
    pub candidates_token_count: Option<i32>,
    pub total_token_count: Option<i32>,
}

/// Error response from API
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}

/// API error details
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
    pub status: String,
}
