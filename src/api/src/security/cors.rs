use axum::{
    http::{HeaderMap, HeaderValue, Method, StatusCode},
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
pub struct CorsLayer {
    config: SecurityConfig,
}

impl CorsLayer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
    config: SecurityConfig,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for CorsMiddleware<S>
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
            let origin = headers.get("origin").and_then(|v| v.to_str().ok());

            // Handle preflight requests
            if method == Method::OPTIONS {
                return Ok(handle_preflight(&config, &headers));
            }

            // Validate origin for actual requests
            if let Some(origin_value) = origin {
                if !is_origin_allowed(origin_value, &config.allowed_origins) {
                    warn!("CORS: Blocked request from unauthorized origin: {}", origin_value);
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(axum::body::Body::from("CORS: Origin not allowed"))
                        .unwrap());
                }
                debug!("CORS: Allowed request from origin: {}", origin_value);
            }

            // Process the request
            let mut response = inner.call(request).await?;
            add_cors_headers(&config, response.headers_mut(), origin);
            Ok(response)
        })
    }
}

fn handle_preflight(config: &SecurityConfig, headers: &HeaderMap) -> Response {
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());
    let requested_method = headers.get("access-control-request-method").and_then(|v| v.to_str().ok());
    let requested_headers = headers.get("access-control-request-headers").and_then(|v| v.to_str().ok());

    // Validate origin
    if let Some(origin_value) = origin {
        if !is_origin_allowed(origin_value, &config.allowed_origins) {
            warn!("CORS Preflight: Blocked request from unauthorized origin: {}", origin_value);
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(axum::body::Body::from("CORS: Origin not allowed"))
                .unwrap();
        }
    }

    // Validate requested method
    let allowed_methods = ["GET", "POST", "PUT", "DELETE"];
    if let Some(method) = requested_method {
        if !allowed_methods.contains(&method) {
            warn!("CORS Preflight: Blocked request with unauthorized method: {}", method);
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(axum::body::Body::from("CORS: Method not allowed"))
                .unwrap();
        }
    }

    // Validate requested headers
    let allowed_headers = ["content-type", "x-requested-with", "x-csrf-token"];
    if let Some(headers_str) = requested_headers {
        for header in headers_str.split(',') {
            let header = header.trim().to_lowercase();
            if !allowed_headers.contains(&header.as_str()) {
                warn!("CORS Preflight: Blocked request with unauthorized header: {}", header);
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(axum::body::Body::from("CORS: Header not allowed"))
                    .unwrap();
            }
        }
    }

    let mut response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap();

    add_preflight_headers(config, response.headers_mut(), origin);
    response
}

fn add_cors_headers(config: &SecurityConfig, headers: &mut HeaderMap, origin: Option<&str>) {
    if let Some(origin_value) = origin {
        if is_origin_allowed(origin_value, &config.allowed_origins) {
            headers.insert("access-control-allow-origin", HeaderValue::from_str(origin_value).unwrap());
            headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
        }
    }

    headers.insert("vary", HeaderValue::from_static("Origin"));
}

fn add_preflight_headers(config: &SecurityConfig, headers: &mut HeaderMap, origin: Option<&str>) {
    if let Some(origin_value) = origin {
        if is_origin_allowed(origin_value, &config.allowed_origins) {
            headers.insert("access-control-allow-origin", HeaderValue::from_str(origin_value).unwrap());
        }
    }

    headers.insert("access-control-allow-methods", HeaderValue::from_static("GET, POST, PUT, DELETE"));
    headers.insert("access-control-allow-headers", HeaderValue::from_static("Content-Type, X-Requested-With, X-CSRF-Token"));
    headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
    headers.insert("access-control-max-age", HeaderValue::from_str(&config.cors_max_age.as_secs().to_string()).unwrap());
    headers.insert("vary", HeaderValue::from_static("Origin"));
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
    async fn test_cors_allowed_origin() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            ..Default::default()
        };
        
        let layer = CorsLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::GET)
            .header("origin", "https://example.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("access-control-allow-origin").unwrap(),
            "https://example.com"
        );
    }

    #[tokio::test]
    async fn test_cors_blocked_origin() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            ..Default::default()
        };
        
        let layer = CorsLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::GET)
            .header("origin", "https://malicious.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_preflight_allowed() {
        let config = SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            ..Default::default()
        };
        
        let layer = CorsLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder()
            .method(Method::OPTIONS)
            .header("origin", "https://example.com")
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", "content-type")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response.headers().get("access-control-allow-origin").unwrap(),
            "https://example.com"
        );
    }
}