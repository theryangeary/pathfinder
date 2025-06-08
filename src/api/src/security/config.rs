use std::env;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub allowed_origins: Vec<String>,
    pub cors_max_age: Duration,
    pub rate_limit_session: u32,
    pub rate_limit_read: u32,
    pub rate_limit_write: u32,
    pub rate_limit_window: Duration,
    pub cookie_max_age: Duration,
    pub request_timeout: Duration,
    pub max_request_size: usize,
    pub strict_referer: bool,
    pub hsts_max_age: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:5173".to_string()],
            cors_max_age: Duration::from_secs(300),
            rate_limit_session: 10,
            rate_limit_read: 200,
            rate_limit_write: 50,
            rate_limit_window: Duration::from_secs(60),
            cookie_max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
            request_timeout: Duration::from_secs(30),
            max_request_size: 1024 * 1024, // 1MB
            strict_referer: true,
            hsts_max_age: 31536000, // 1 year
        }
    }
}

impl SecurityConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(origins) = env::var("ALLOWED_ORIGINS") {
            config.allowed_origins = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        if let Ok(max_age) = env::var("CORS_MAX_AGE") {
            if let Ok(seconds) = max_age.parse::<u64>() {
                config.cors_max_age = Duration::from_secs(seconds);
            }
        }

        if let Ok(limit) = env::var("RATE_LIMIT_SESSION") {
            if let Ok(value) = limit.parse::<u32>() {
                config.rate_limit_session = value;
            }
        }

        if let Ok(limit) = env::var("RATE_LIMIT_READ") {
            if let Ok(value) = limit.parse::<u32>() {
                config.rate_limit_read = value;
            }
        }

        if let Ok(limit) = env::var("RATE_LIMIT_WRITE") {
            if let Ok(value) = limit.parse::<u32>() {
                config.rate_limit_write = value;
            }
        }

        if let Ok(window) = env::var("RATE_LIMIT_WINDOW") {
            if let Ok(seconds) = window.parse::<u64>() {
                config.rate_limit_window = Duration::from_secs(seconds);
            }
        }

        if let Ok(max_age) = env::var("COOKIE_MAX_AGE") {
            if let Ok(seconds) = max_age.parse::<u64>() {
                config.cookie_max_age = Duration::from_secs(seconds);
            }
        }

        if let Ok(timeout) = env::var("REQUEST_TIMEOUT") {
            if let Ok(seconds) = timeout.parse::<u64>() {
                config.request_timeout = Duration::from_secs(seconds);
            }
        }

        if let Ok(size) = env::var("MAX_REQUEST_SIZE") {
            if let Ok(bytes) = size.parse::<usize>() {
                config.max_request_size = bytes;
            }
        }

        if let Ok(strict) = env::var("STRICT_REFERER") {
            config.strict_referer = strict.to_lowercase() == "true";
        }

        if let Ok(max_age) = env::var("HSTS_MAX_AGE") {
            if let Ok(seconds) = max_age.parse::<u64>() {
                config.hsts_max_age = seconds;
            }
        }

        config
    }
}