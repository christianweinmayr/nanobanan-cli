mod types;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::path::Path;
use tokio::fs;

pub use types::*;

use crate::config::Config;
use crate::core::{BananaError, GenerateParams, Job, JobStatus};
use crate::http_client::HTTP_CLIENT;

/// Gemini API client
pub struct GeminiClient {
    api_key: String,
    base_url: String,
}

impl GeminiClient {
    /// Create a new client from config
    pub fn from_config(config: &Config) -> Result<Self, BananaError> {
        let api_key = config
            .api_key()
            .ok_or(BananaError::MissingApiKey)?
            .to_string();

        Ok(Self {
            api_key,
            base_url: config.api.base_url.clone(),
        })
    }

    /// Generate images from a prompt
    pub async fn generate(&self, params: &GenerateParams) -> Result<GenerateResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, params.model, self.api_key
        );

        let request = self.build_generate_request(params);

        tracing::debug!("Sending generate request to: {}", url);
        tracing::debug!("Request body: {}", serde_json::to_string_pretty(&request)?);

        let response = HTTP_CLIENT
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Gemini API")?;

        let status = response.status();
        let body = response.text().await?;

        tracing::debug!("Response status: {}", status);
        tracing::debug!("Response body: {}", body);

        if !status.is_success() {
            let error: ApiErrorResponse = serde_json::from_str(&body)
                .unwrap_or_else(|_| ApiErrorResponse {
                    error: ApiError {
                        code: status.as_u16() as i32,
                        message: body.clone(),
                        status: status.to_string(),
                    },
                });
            return Err(BananaError::ApiError {
                message: error.error.message,
                source: None,
            }
            .into());
        }

        let response: GenerateResponse = serde_json::from_str(&body)
            .context("Failed to parse Gemini API response")?;

        Ok(response)
    }

    /// Build the API request body
    fn build_generate_request(&self, params: &GenerateParams) -> GenerateRequest {
        let mut parts = vec![ContentPart::Text {
            text: params.prompt.clone(),
        }];

        // Add reference image if present (for editing)
        if let (Some(data), Some(mime_type)) = (&params.reference_image, &params.reference_mime_type) {
            parts.insert(
                0,
                ContentPart::InlineData {
                    inlineData: InlineData {
                        mime_type: mime_type.clone(),
                        data: data.clone(),
                    },
                },
            );
        }

        GenerateRequest {
            contents: vec![Content {
                parts,
                role: None,
            }],
            generation_config: Some(GenerationConfig {
                response_modalities: Some(vec!["TEXT".to_string(), "IMAGE".to_string()]),
                image_config: Some(ImageConfig {
                    aspect_ratio: Some(params.aspect_ratio.clone()),
                }),
            }),
            safety_settings: None,
        }
    }

    /// Extract images from response and update job
    pub fn process_response(&self, job: &mut Job, response: GenerateResponse) -> Result<()> {
        let mut image_index = 0u8;

        for candidate in response.candidates.unwrap_or_default() {
            // Check for refusal/recitation before processing content
            if let Some(reason) = &candidate.finish_reason {
                if reason != "STOP" && reason != "MAX_TOKENS" {
                    let message = candidate
                        .finish_message
                        .as_deref()
                        .unwrap_or("Image generation was refused by the API");
                    tracing::warn!("Generation refused: {} - {}", reason, message);
                    job.set_failed(message);
                    return Err(
                        BananaError::GenerationFailed(message.to_string()).into()
                    );
                }
            }

            if let Some(content) = candidate.content {
                for part in content.parts {
                    match part {
                        ContentPart::InlineData { inlineData } => {
                            job.add_image(image_index, inlineData.data, inlineData.mime_type);
                            image_index += 1;
                        }
                        ContentPart::Text { text } => {
                            tracing::debug!("Response text: {}", text);
                        }
                    }
                }
            }
        }

        if job.images.is_empty() {
            job.set_failed("No images generated");
            return Err(BananaError::GenerationFailed("No images in response".to_string()).into());
        }

        job.set_completed();
        Ok(())
    }

    /// Download images from job to disk
    pub async fn download_images(&self, job: &mut Job, output_dir: &Path) -> Result<Vec<String>> {
        fs::create_dir_all(output_dir).await?;

        let mut paths = Vec::new();

        for image in &mut job.images {
            if let Some(data) = &image.data {
                let ext = match image.mime_type.as_str() {
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    "image/webp" => "webp",
                    _ => "png",
                };

                let filename = format!("{}_{}.{}", job.id, image.index, ext);
                let path = output_dir.join(&filename);

                let bytes = BASE64
                    .decode(data)
                    .context("Failed to decode base64 image")?;

                fs::write(&path, &bytes).await?;

                image.path = Some(path.to_string_lossy().to_string());
                image.data = None; // Clear base64 data after saving
                paths.push(path.to_string_lossy().to_string());

                tracing::info!("Saved image to: {}", path.display());
            }
        }

        Ok(paths)
    }
}

/// Load an image file and encode as base64
pub async fn load_image_base64(path: &Path) -> Result<(String, String)> {
    let data = fs::read(path).await?;
    let base64_data = BASE64.encode(&data);

    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/png",
    };

    Ok((base64_data, mime_type.to_string()))
}
