use axum::http::HeaderMap;
use std::net::IpAddr;
use std::str::FromStr;

pub fn extract_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // Try X-Forwarded-For first (for load balancers/proxies)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP in the comma-separated list
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = IpAddr::from_str(first_ip.trim()) {
                    return Some(ip);
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            if let Ok(ip) = IpAddr::from_str(real_ip_str) {
                return Some(ip);
            }
        }
    }

    // Fallback to connecting IP (would need to be passed through)
    None
}

pub fn is_origin_allowed(origin: &str, allowed_origins: &[String]) -> bool {
    allowed_origins.iter().any(|allowed| {
        if allowed == "*" {
            return true;
        }

        // Exact match
        if origin == allowed {
            return true;
        }

        // Subdomain match (if allowed origin starts with .)
        if let Some(domain) = allowed.strip_prefix('.') {
            return origin.ends_with(domain);
        }

        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_client_ip() {
        let mut headers = HeaderMap::new();

        // Test X-Forwarded-For
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.168.1.1, 10.0.0.1"),
        );
        assert_eq!(
            extract_client_ip(&headers),
            Some(IpAddr::from_str("192.168.1.1").unwrap())
        );

        // Test X-Real-IP
        headers.clear();
        headers.insert("x-real-ip", HeaderValue::from_static("172.16.0.1"));
        assert_eq!(
            extract_client_ip(&headers),
            Some(IpAddr::from_str("172.16.0.1").unwrap())
        );

        // Test no headers
        headers.clear();
        assert_eq!(extract_client_ip(&headers), None);
    }

    #[test]
    fn test_is_origin_allowed() {
        let allowed = vec![
            "https://example.com".to_string(),
            "http://localhost:3000".to_string(),
            ".example.org".to_string(),
        ];

        assert!(is_origin_allowed("https://example.com", &allowed));
        assert!(is_origin_allowed("http://localhost:3000", &allowed));
        assert!(is_origin_allowed("https://sub.example.org", &allowed));
        assert!(is_origin_allowed("https://api.example.org", &allowed));
        assert!(!is_origin_allowed("https://malicious.com", &allowed));
        assert!(!is_origin_allowed("https://example.com.evil.com", &allowed));
    }
}
