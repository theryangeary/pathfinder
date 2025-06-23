#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use crate::security::SecurityConfig;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        Router,
    };
    use std::time::Duration;
    use tower::ServiceExt;

    fn create_test_config() -> SecurityConfig {
        SecurityConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            cors_max_age: Duration::from_secs(300),
            rate_limit_session: 5,
            rate_limit_read: 10,
            rate_limit_write: 3,
            rate_limit_window: Duration::from_secs(60),
            cookie_max_age: Duration::from_secs(3600),
            request_timeout: Duration::from_secs(30),
            max_request_size: 1024,
            strict_referer: true,
            hsts_max_age: 31536000,
        }
    }

    async fn test_handler() -> axum::response::Response {
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("test response"))
            .unwrap()
    }

    fn create_test_router() -> Router {
        let config = create_test_config();

        Router::new()
            .route("/test", axum::routing::get(test_handler))
            .route("/test", axum::routing::post(test_handler))
            .route("/api/test", axum::routing::get(test_handler))
            .route("/api/user", axum::routing::post(test_handler))
            .route("/health", axum::routing::get(test_handler))
            .layer(tower_http::limit::RequestBodyLimitLayer::new(
                config.max_request_size,
            ))
            .layer(tower_http::timeout::TimeoutLayer::new(
                config.request_timeout,
            ))
            .layer(crate::security::rate_limit::RateLimitLayer::new(
                config.clone(),
            ))
            .layer(crate::security::cors::CorsLayer::new(config.clone()))
            .layer(crate::security::referer::RefererLayer::new(config.clone()))
            .layer(crate::security::session::SessionLayer::new(config.clone()))
            .layer(crate::security::session::cookie_layer())
            .layer(crate::security::headers::SecurityHeadersLayer::new(
                config.clone(),
            ))
    }

    #[tokio::test]
    async fn test_security_middleware_integration() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("origin", "https://example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed with all security headers
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("strict-transport-security"));
        assert_eq!(
            headers.get("access-control-allow-origin").unwrap(),
            "https://example.com"
        );
    }

    #[tokio::test]
    async fn test_cors_blocks_unauthorized_origin() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("origin", "https://malicious.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_referer_validation_blocks_invalid_referer() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://example.com")
            .header("referer", "https://malicious.com/attack")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_rate_limiting_works() {
        let app = create_test_router();

        // Make requests up to the limit (3 for write operations)
        for _ in 0..3 {
            let request = Request::builder()
                .method(Method::POST)
                .uri("/test")
                .header("origin", "https://example.com")
                .header("referer", "https://example.com/page")
                .header("x-forwarded-for", "192.168.1.100") // Consistent IP
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Next request should be rate limited
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://example.com")
            .header("referer", "https://example.com/page")
            .header("x-forwarded-for", "192.168.1.100") // Same IP
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key("retry-after"));
    }

    #[tokio::test]
    async fn test_health_check_bypasses_rate_limiting() {
        let app = create_test_router();

        // Make many requests to health endpoint - should not be rate limited
        for _ in 0..10 {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_request_size_limit() {
        let app = create_test_router();

        // Create a request body larger than the limit (1024 bytes)
        let large_body = "x".repeat(2048);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://example.com")
            .header("referer", "https://example.com/page")
            .header("content-type", "application/json")
            .header("content-length", "2048")
            .body(Body::from(large_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_session_creation_and_validation() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/test")
            .header("origin", "https://example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check if session cookie was set
        let set_cookie_header = response.headers().get("set-cookie");
        assert!(set_cookie_header.is_some());

        let cookie_value = set_cookie_header.unwrap().to_str().unwrap();
        assert!(cookie_value.contains("session_id="));
        assert!(cookie_value.contains("HttpOnly"));
        assert!(cookie_value.contains("Secure"));
        assert!(cookie_value.contains("SameSite=Lax"));
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("origin", "https://example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let headers = response.headers();

        // Check all required security headers
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
        assert_eq!(headers.get("x-dns-prefetch-control").unwrap(), "off");
    }

    #[tokio::test]
    async fn test_preflight_cors_request() {
        let app = create_test_router();

        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri("/test")
            .header("origin", "https://example.com")
            .header("access-control-request-method", "POST")
            .header("access-control-request-headers", "content-type")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let headers = response.headers();
        assert_eq!(
            headers.get("access-control-allow-origin").unwrap(),
            "https://example.com"
        );
        assert!(headers
            .get("access-control-allow-methods")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("POST"));
        assert_eq!(
            headers.get("access-control-allow-credentials").unwrap(),
            "true"
        );
    }
}
