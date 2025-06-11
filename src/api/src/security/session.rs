use axum::{
    response::Response,
};
use std::convert::Infallible;
use tower::{Layer, Service};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use tracing::{debug, info};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose};

use crate::security::SecurityConfig;

#[derive(Clone)]
pub struct SessionLayer {
    config: SecurityConfig,
}

impl SessionLayer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S> {
    inner: S,
    config: SecurityConfig,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for SessionMiddleware<S>
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

    fn call(&mut self, mut request: axum::http::Request<axum::body::Body>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract cookies from request
            let cookies = request.extensions().get::<Cookies>().cloned();
            let session_id = extract_session_id(&cookies);
            
            let (should_create_session, new_session_id) = match session_id {
                Some(id) => {
                    if is_valid_session_id(&id) {
                        debug!("Valid session found: {}", mask_session_id(&id));
                        (false, id)
                    } else {
                        debug!("Invalid session ID, creating new session");
                        (true, generate_session_id())
                    }
                }
                None => {
                    debug!("No session found, creating new session");
                    (true, generate_session_id())
                }
            };

            // Add session ID to request extensions for use by handlers
            request.extensions_mut().insert(SessionInfo {
                session_id: new_session_id.clone(),
                is_new: should_create_session,
            });

            // Process the request
            let response = inner.call(request).await?;

            // Set session cookie if needed
            if should_create_session {
                if let Some(cookies) = cookies {
                    set_session_cookie(&config, &cookies, &new_session_id);
                    info!("Created new session: {}", mask_session_id(&new_session_id));
                }
            }

            Ok(response)
        })
    }
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub session_id: String,
    pub is_new: bool,
}

fn extract_session_id(cookies: &Option<Cookies>) -> Option<String> {
    cookies.as_ref()?.get("session_id").map(|cookie| cookie.value().to_string())
}

fn generate_session_id() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32]; // 256 bits of entropy
    rng.fill(&mut bytes).expect("Failed to generate random bytes");
    general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn is_valid_session_id(session_id: &str) -> bool {
    // Basic validation - check if it's a valid base64 string of expected length
    if session_id.len() != 43 { // 32 bytes base64-encoded without padding
        return false;
    }
    
    // Check if it's valid base64
    general_purpose::URL_SAFE_NO_PAD.decode(session_id).is_ok()
}

fn set_session_cookie(config: &SecurityConfig, cookies: &Cookies, session_id: &str) {
    let mut cookie = Cookie::new("session_id", session_id.to_string());
    
    // Security attributes
    cookie.set_http_only(true);
    cookie.set_secure(true); // Only send over HTTPS
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax); // CSRF protection
    cookie.set_path("/api"); // Restrict to API endpoints
    
    // Set expiration
    let max_age = tower_cookies::cookie::time::Duration::seconds(config.cookie_max_age.as_secs() as i64);
    cookie.set_max_age(max_age);
    
    cookies.add(cookie);
}

fn mask_session_id(session_id: &str) -> String {
    if session_id.len() > 8 {
        format!("{}...{}", &session_id[..4], &session_id[session_id.len()-4..])
    } else {
        "****".to_string()
    }
}

// Layer function to add cookie manager
pub fn cookie_layer() -> CookieManagerLayer {
    CookieManagerLayer::new()
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

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 43); // 32 bytes base64-encoded without padding
        assert!(is_valid_session_id(&id1));
        assert!(is_valid_session_id(&id2));
    }

    #[test]
    fn test_is_valid_session_id() {
        let valid_id = generate_session_id();
        assert!(is_valid_session_id(&valid_id));
        
        assert!(!is_valid_session_id("too_short"));
        assert!(!is_valid_session_id("invalid@characters!"));
        assert!(!is_valid_session_id("")); 
    }

    #[test]
    fn test_mask_session_id() {
        let id = "abcdefghijklmnopqrstuvwxyz1234567890ABCDEF";
        let masked = mask_session_id(&id);
        assert_eq!(masked, "abcd...CDEF");
        
        let short_id = "abc";
        let masked_short = mask_session_id(&short_id);
        assert_eq!(masked_short, "****");
    }

    #[tokio::test]
    async fn test_session_middleware_creates_new_session() {
        let config = SecurityConfig::default();
        let layer = SessionLayer::new(config);
        
        // Create a service that wraps the test service with cookie manager
        let service_with_cookies = tower::ServiceBuilder::new()
            .layer(cookie_layer())
            .layer(layer)
            .service(tower::service_fn(|req: Request<axum::body::Body>| async move {
                // Check if session info was added
                let session_info = req.extensions().get::<SessionInfo>();
                assert!(session_info.is_some());
                assert!(session_info.unwrap().is_new);
                Ok(test_service().await)
            }));

        let request = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service_with_cookies.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}