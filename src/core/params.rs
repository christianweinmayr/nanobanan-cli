use serde::{Deserialize, Serialize};

/// Parameters for image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateParams {
    /// The prompt for image generation
    pub prompt: String,

    /// Aspect ratio (e.g., "1:1", "16:9", "9:16")
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,

    /// Image size: "1K", "2K", "4K" (4K only for Gemini 3 Pro)
    #[serde(default = "default_size")]
    pub size: String,

    /// Model to use
    #[serde(default = "default_model")]
    pub model: String,

    /// Number of images to generate (1-4)
    #[serde(default = "default_num_images")]
    pub num_images: u8,

    /// Optional seed for reproducibility
    pub seed: Option<i64>,

    /// Optional negative prompt (what to avoid)
    pub negative_prompt: Option<String>,

    /// Reference image for editing (base64 encoded)
    pub reference_image: Option<String>,

    /// Reference image mime type
    pub reference_mime_type: Option<String>,
}

fn default_aspect_ratio() -> String {
    "1:1".to_string()
}

fn default_size() -> String {
    "1K".to_string()
}

fn default_model() -> String {
    "gemini-3-pro-image-preview".to_string()
}

fn default_num_images() -> u8 {
    1
}

impl Default for GenerateParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            aspect_ratio: default_aspect_ratio(),
            size: default_size(),
            model: default_model(),
            num_images: 1,
            seed: None,
            negative_prompt: None,
            reference_image: None,
            reference_mime_type: None,
        }
    }
}

impl GenerateParams {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    pub fn with_aspect_ratio(mut self, ar: impl Into<String>) -> Self {
        self.aspect_ratio = ar.into();
        self
    }

    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = size.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_num_images(mut self, num: u8) -> Self {
        self.num_images = num.clamp(1, 4);
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_negative_prompt(mut self, neg: impl Into<String>) -> Self {
        self.negative_prompt = Some(neg.into());
        self
    }

    pub fn with_reference_image(mut self, base64_data: String, mime_type: String) -> Self {
        self.reference_image = Some(base64_data);
        self.reference_mime_type = Some(mime_type);
        self
    }

    /// Check if this is an edit request (has reference image)
    pub fn is_edit(&self) -> bool {
        self.reference_image.is_some()
    }
}
