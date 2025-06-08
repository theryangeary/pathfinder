use axum::{
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};
use std::convert::Infallible;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use tracing::{warn, debug};

use crate::security::{SecurityConfig, utils::is_origin_allowed};

#[derive(Clone)]
pub struct RefererLayer {
    config: SecurityConfig,
}

impl RefererLayer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for RefererLayer {
    type Service = RefererMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RefererMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RefererMiddleware<S> {
    inner: S,
    config: SecurityConfig,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for RefererMiddleware<S>
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
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let method = request.method().clone();
            let headers = request.headers().clone();
            let uri = request.uri().clone();

            // Only check referer for state-changing operations
            if matches!(method, Method::POST | Method::PUT | Method::DELETE) {
                if let Err(response) = validate_referer(&config, &headers, &uri) {
                    return Ok(response);
                }
            }

            // Process the request
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}

fn validate_referer(
    config: &SecurityConfig,
    headers: &HeaderMap,
    uri: &axum::http::Uri,
) -> Result<(), Response> {
    let referer = headers.get("referer").and_then(|v| v.to_str().ok());
    
    // Skip referer validation for specific endpoints if configured
    let path = uri.path();
    if should_skip_referer_check(path) {
        debug!("Skipping referer check for path: {}", path);
        return Ok(());
    }

    match referer {
        Some(referer_value) => {
            // Extract origin from referer URL
            if let Ok(referer_url) = url::Url::parse(referer_value) {
                let referer_origin = format!("{}://{}", referer_url.scheme(), 
                    referer_url.host_str().unwrap_or(""));
                
                // Add port if it's not the default for the scheme
                let referer_origin = if let Some(port) = referer_url.port() {
                    if (referer_url.scheme() == "https" && port != 443) ||
                       (referer_url.scheme() == "http" && port != 80) {
                        format!("{}:{}", referer_origin, port)
                    } else {
                        referer_origin
                    }
                } else {
                    referer_origin
                };

                if is_origin_allowed(&referer_origin, &config.allowed_origins) {
                    debug!("Referer validation passed for: {}", referer_value);
                    Ok(())
                } else {
                    warn!("Referer validation failed - invalid referer: {}", referer_value);
                    log_suspicious_request(headers, uri, "Invalid referer");
                    Err(create_referer_error_response("Invalid referer"))
                }
            } else {
                warn!("Referer validation failed - malformed referer: {}", referer_value);
                log_suspicious_request(headers, uri, "Malformed referer");
                Err(create_referer_error_response("Malformed referer"))
            }
        }
        None => {
            if config.strict_referer {
                warn!("Referer validation failed - missing referer for {}", path);
                log_suspicious_request(headers, uri, "Missing referer");
                Err(create_referer_error_response("Missing referer"))
            } else {
                debug!("Allowing request without referer (strict_referer=false)");
                Ok(())
            }
        }
    }
}

fn should_skip_referer_check(path: &str) -> bool {
    // Skip referer check for API endpoints that might be called directly
    matches!(path, 
        "/health" | 
        "/api/health" |
        "/metrics" |
        "/api/metrics"
    )
}

fn log_suspicious_request(headers: &HeaderMap, uri: &axum::http::Uri, reason: &str) {
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    let x_forwarded_for = headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    warn!(
        "Suspicious request blocked: {} | Path: {} | User-Agent: {} | X-Forwarded-For: {}",
        reason,
        uri.path(),
        user_agent,
        x_forwarded_for
    );
}

fn create_referer_error_response(message: &str) -> Response {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(format!(
            r#"{{"error": "Referer validation failed", "message": "{}"}}"#,
            message
        )))
        .unwrap()
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

    #[tokio::test]
    async fn test_referer_validation_passes() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            strict_referer: true,
            ..Default::default()
        };
        
        let layer = RefererLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/submit")
            .header("referer", "https://example.com/game")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_referer_validation_fails() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            strict_referer: true,
            ..Default::default()
        };
        
        let layer = RefererLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/submit")
            .header("referer", "https://malicious.com/attack")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_requests_skip_referer_check() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            strict_referer: true,
            ..Default::default()
        };
        
        let layer = RefererLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        // GET requests should not be checked for referer
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/game/date/2025-01-01")
            .header("referer", "https://malicious.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_referer_with_strict_false() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            strict_referer: false,
            ..Default::default()
        };
        
        let layer = RefererLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/submit")
            // No referer header
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_check_skips_referer() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            strict_referer: true,
            ..Default::default()
        };
        
        let layer = RefererLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .header("referer", "https://malicious.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}