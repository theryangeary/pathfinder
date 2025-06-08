use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};
use governor::{
    clock::{DefaultClock, Clock},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};
use tracing::{debug, warn};
use std::num::NonZeroU32;

use crate::security::{SecurityConfig, utils::extract_client_ip};

// Simplified rate limiter - we'll implement a basic in-memory HashMap-based solution
type SimpleRateLimiter = Arc<RwLock<HashMap<IpAddr, (Instant, u32)>>>;

#[derive(Clone)]
pub struct RateLimitLayer {
    config: SecurityConfig,
}

impl RateLimitLayer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let session_limiter = Arc::new(RwLock::new(HashMap::new()));
        let read_limiter = Arc::new(RwLock::new(HashMap::new()));
        let write_limiter = Arc::new(RwLock::new(HashMap::new()));

        // Spawn cleanup task
        let session_limiter_clone = session_limiter.clone();
        let read_limiter_clone = read_limiter.clone();
        let write_limiter_clone = write_limiter.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Cleanup every 5 minutes
            loop {
                interval.tick().await;
                cleanup_old_entries_simple(&session_limiter_clone).await;
                cleanup_old_entries_simple(&read_limiter_clone).await;
                cleanup_old_entries_simple(&write_limiter_clone).await;
            }
        });

        RateLimitMiddleware {
            inner,
            config: self.config.clone(),
            session_limiter,
            read_limiter,
            write_limiter,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    config: SecurityConfig,
    session_limiter: SimpleRateLimiter,
    read_limiter: SimpleRateLimiter,
    write_limiter: SimpleRateLimiter,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for RateLimitMiddleware<S>
where
    S: Service<axum::http::Request<axum::body::Body>, Response = Response, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: axum::http::Request<axum::body::Body>) -> Self::Future {
        let config = self.config.clone();
        let session_limiter = self.session_limiter.clone();
        let read_limiter = self.read_limiter.clone();
        let write_limiter = self.write_limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let headers = request.headers();
            let uri = request.uri();
            let method = request.method();

            // Extract client IP
            let client_ip = extract_client_ip(headers)
                .or_else(|| {
                    request.extensions().get::<ConnectInfo<SocketAddr>>()
                        .map(|connect_info| connect_info.0.ip())
                })
                .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1])); // Fallback to localhost

            // Determine rate limit type based on endpoint
            let (limiter, limit) = match determine_endpoint_type(uri, method.as_str()) {
                EndpointType::Session => (session_limiter, config.rate_limit_session),
                EndpointType::Read => (read_limiter, config.rate_limit_read),
                EndpointType::Write => (write_limiter, config.rate_limit_write),
                EndpointType::Health => {
                    // Skip rate limiting for health checks
                    let response = inner.call(request).await?;
                    return Ok(response);
                }
            };

            // Check rate limit
            let now = Instant::now();
            let allowed = {
                let mut limiter_guard = limiter.write().unwrap();
                let entry = limiter_guard.entry(client_ip).or_insert((now, 0));
                
                // Reset counter if window has passed
                if now.duration_since(entry.0) >= config.rate_limit_window {
                    entry.0 = now;
                    entry.1 = 0;
                }
                
                // Check if under limit
                if entry.1 < limit {
                    entry.1 += 1;
                    true
                } else {
                    false
                }
            };

            if allowed {
                debug!("Rate limit OK for IP: {}", client_ip);
                let mut response = inner.call(request).await?;
                add_rate_limit_headers_simple(&config, response.headers_mut(), limit);
                Ok(response)
            } else {
                warn!("Rate limit exceeded for IP: {}", client_ip);
                
                let retry_after = config.rate_limit_window.as_secs();
                let mut response = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("retry-after", retry_after.to_string())
                    .body(axum::body::Body::from("Rate limit exceeded"))
                    .unwrap();
                
                add_rate_limit_headers_simple(&config, response.headers_mut(), limit);
                Ok(response)
            }
        })
    }
}

#[derive(Debug, PartialEq)]
enum EndpointType {
    Session,
    Read,
    Write,
    Health,
}

fn determine_endpoint_type(uri: &Uri, method: &str) -> EndpointType {
    let path = uri.path();

    // Health check endpoint
    if path == "/health" || path == "/api/health" {
        return EndpointType::Health;
    }

    // Session endpoints
    if path == "/api/user" && method == "POST" {
        return EndpointType::Session;
    }

    // Write operations
    if matches!(method, "POST" | "PUT" | "DELETE") {
        if path.contains("/submit") || path.contains("/validate") {
            return EndpointType::Write;
        }
    }

    // Default to read operations
    EndpointType::Read
}

fn add_rate_limit_headers_simple(
    config: &SecurityConfig,
    headers: &mut HeaderMap,
    limit: u32,
) {
    let reset_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + config.rate_limit_window.as_secs();

    headers.insert("x-ratelimit-limit", axum::http::HeaderValue::from_str(&limit.to_string()).unwrap());
    headers.insert("x-ratelimit-remaining", axum::http::HeaderValue::from_str(&(limit / 2).to_string()).unwrap());
    headers.insert("x-ratelimit-reset", axum::http::HeaderValue::from_str(&reset_time.to_string()).unwrap());
}

async fn cleanup_old_entries_simple(limiter: &SimpleRateLimiter) {
    let cutoff = Instant::now() - Duration::from_secs(3600); // Remove entries older than 1 hour
    
    let mut to_remove = Vec::new();
    {
        let reader = limiter.read().unwrap();
        for (ip, (last_seen, _count)) in reader.iter() {
            if *last_seen < cutoff {
                to_remove.push(*ip);
            }
        }
    }

    if !to_remove.is_empty() {
        let mut writer = limiter.write().unwrap();
        for ip in to_remove {
            writer.remove(&ip);
        }
        debug!("Cleaned up {} old rate limit entries", writer.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Method};
    use tower::ServiceExt;

    async fn test_service() -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .body(axum::body::Body::from("test"))
            .unwrap()
    }

    #[test]
    fn test_determine_endpoint_type() {
        assert_eq!(
            determine_endpoint_type(&Uri::from_static("/api/user"), "POST"),
            EndpointType::Session
        );
        assert_eq!(
            determine_endpoint_type(&Uri::from_static("/api/submit"), "POST"),
            EndpointType::Write
        );
        assert_eq!(
            determine_endpoint_type(&Uri::from_static("/api/validate"), "POST"),
            EndpointType::Write
        );
        assert_eq!(
            determine_endpoint_type(&Uri::from_static("/api/game/date/2025-01-01"), "GET"),
            EndpointType::Read
        );
        assert_eq!(
            determine_endpoint_type(&Uri::from_static("/health"), "GET"),
            EndpointType::Health
        );
    }

    #[tokio::test]
    async fn test_rate_limit_allows_normal_requests() {
        let config = SecurityConfig {
            rate_limit_read: 100,
            ..Default::default()
        };
        
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/game/date/2025-01-01")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-ratelimit-limit"));
    }

    #[tokio::test]
    async fn test_health_check_bypass() {
        let config = SecurityConfig {
            rate_limit_read: 1, // Very low limit
            ..Default::default()
        };
        
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        // Health check should always pass
        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}