use axum::{http::HeaderMap, response::Response};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::security::SecurityConfig;

#[derive(Clone)]
pub struct SecurityHeadersLayer {
    config: SecurityConfig,
}

impl SecurityHeadersLayer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersMiddleware<S> {
    inner: S,
    config: SecurityConfig,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for SecurityHeadersMiddleware<S>
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
            // Process the request
            let mut response = inner.call(request).await?;

            // Add security headers
            add_security_headers(&config, response.headers_mut());

            Ok(response)
        })
    }
}

fn add_security_headers(config: &SecurityConfig, headers: &mut HeaderMap) {
    // X-Content-Type-Options: Prevents MIME type sniffing
    headers.insert(
        "x-content-type-options",
        axum::http::HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options: Prevents clickjacking
    headers.insert(
        "x-frame-options",
        axum::http::HeaderValue::from_static("DENY"),
    );

    // X-XSS-Protection: Legacy XSS protection (still useful for older browsers)
    headers.insert(
        "x-xss-protection",
        axum::http::HeaderValue::from_static("1; mode=block"),
    );

    // Strict-Transport-Security: Enforces HTTPS
    let hsts_value = format!("max-age={}; includeSubDomains", config.hsts_max_age);
    if let Ok(hsts_header) = axum::http::HeaderValue::from_str(&hsts_value) {
        headers.insert("strict-transport-security", hsts_header);
    }

    // Content-Security-Policy: Prevents various attacks
    // This is a restrictive policy - adjust based on your needs
    let csp_policy = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
    if let Ok(csp_header) = axum::http::HeaderValue::from_str(csp_policy) {
        headers.insert("content-security-policy", csp_header);
    }

    // Referrer-Policy: Controls referrer information
    headers.insert(
        "referrer-policy",
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy: Controls access to browser features
    let permissions_policy = "geolocation=(), microphone=(), camera=()";
    if let Ok(permissions_header) = axum::http::HeaderValue::from_str(permissions_policy) {
        headers.insert("permissions-policy", permissions_header);
    }

    // X-Permitted-Cross-Domain-Policies: Controls Adobe Flash and Acrobat cross-domain policies
    headers.insert(
        "x-permitted-cross-domain-policies",
        axum::http::HeaderValue::from_static("none"),
    );

    // Cache-Control: Prevent caching of sensitive responses
    // Note: This is applied to all responses - you might want to be more selective
    headers.insert(
        "cache-control",
        axum::http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );

    // Pragma: Legacy cache control
    headers.insert("pragma", axum::http::HeaderValue::from_static("no-cache"));

    // X-DNS-Prefetch-Control: Controls DNS prefetching
    headers.insert(
        "x-dns-prefetch-control",
        axum::http::HeaderValue::from_static("off"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn test_service() -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .body(axum::body::Body::from("test"))
            .unwrap()
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let config = SecurityConfig::default();
        let layer = SecurityHeadersLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder().body(axum::body::Body::empty()).unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // Check that all security headers are present
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("content-security-policy"));
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert!(headers.contains_key("permissions-policy"));
        assert_eq!(
            headers.get("x-permitted-cross-domain-policies").unwrap(),
            "none"
        );
        assert_eq!(
            headers.get("cache-control").unwrap(),
            "no-cache, no-store, must-revalidate"
        );
        assert_eq!(headers.get("pragma").unwrap(), "no-cache");
        assert_eq!(headers.get("x-dns-prefetch-control").unwrap(), "off");
    }

    #[tokio::test]
    async fn test_hsts_header_format() {
        let config = SecurityConfig {
            hsts_max_age: 31536000, // 1 year
            ..Default::default()
        };

        let layer = SecurityHeadersLayer::new(config);
        let mut service = layer.layer(tower::service_fn(|_| async { Ok(test_service().await) }));

        let request = Request::builder().body(axum::body::Body::empty()).unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();

        let hsts_header = response.headers().get("strict-transport-security").unwrap();
        assert_eq!(hsts_header, "max-age=31536000; includeSubDomains");
    }

    #[test]
    fn test_add_security_headers() {
        let config = SecurityConfig::default();
        let mut headers = HeaderMap::new();

        add_security_headers(&config, &mut headers);

        // Verify all expected headers are present
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("x-xss-protection"));
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("content-security-policy"));
        assert!(headers.contains_key("referrer-policy"));
        assert!(headers.contains_key("permissions-policy"));
        assert!(headers.contains_key("x-permitted-cross-domain-policies"));
        assert!(headers.contains_key("cache-control"));
        assert!(headers.contains_key("pragma"));
        assert!(headers.contains_key("x-dns-prefetch-control"));
    }
}
