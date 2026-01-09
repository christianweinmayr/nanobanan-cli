use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

/// Shared HTTP client with connection pooling
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(120)) // Longer timeout for image generation
        .pool_max_idle_per_host(5)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
});
