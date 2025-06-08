# Security Implementation

This document describes the comprehensive security features implemented in the word game backend API.

## Overview

The API implements a layered security approach with the following features:
- CORS configuration with origin validation
- IP-based rate limiting with different limits per endpoint type
- Referer header validation for state-changing operations
- Secure cookie/session management
- Comprehensive security headers
- Request size and timeout limits

## Security Middleware Stack

The middleware is applied in the following order (executed in reverse order):

1. **Request Body Limit** - Limits request size to prevent DoS attacks
2. **Request Timeout** - Prevents slow loris attacks
3. **Rate Limiting** - IP-based rate limiting with cleanup
4. **CORS** - Origin validation and preflight handling
5. **Referer Validation** - Validates referer header for state-changing operations
6. **Cookie Management** - Handles secure session cookies
7. **Session Management** - Creates and validates sessions
8. **Security Headers** - Adds comprehensive security headers

## Configuration

All security settings are configurable via environment variables. See `.env.example` for detailed configuration options.

### Key Configuration Options

- `ALLOWED_ORIGINS`: Comma-separated list of allowed origins
- `RATE_LIMIT_SESSION`: Rate limit for session creation (per minute per IP)
- `RATE_LIMIT_READ`: Rate limit for read operations (per minute per IP)
- `RATE_LIMIT_WRITE`: Rate limit for write operations (per minute per IP)
- `STRICT_REFERER`: Whether to require referer header for state-changing operations
- `COOKIE_MAX_AGE`: Session cookie expiration time
- `REQUEST_TIMEOUT`: Maximum request processing time
- `MAX_REQUEST_SIZE`: Maximum request body size

## Rate Limiting

### Endpoint Classification

- **Session endpoints** (`/api/user` POST): 10 requests/minute per IP (default)
- **Read operations** (GET requests): 200 requests/minute per IP (default)
- **Write operations** (POST/PUT/DELETE): 50 requests/minute per IP (default)
- **Health checks** (`/health`): Exempt from rate limiting

### IP Extraction

The rate limiter extracts client IPs in the following order:
1. `X-Forwarded-For` header (first IP in list)
2. `X-Real-IP` header
3. Connection IP (fallback)

### Cleanup

Old rate limit entries are automatically cleaned up every 5 minutes to prevent memory leaks.

## CORS Configuration

### Features

- Origin validation against whitelist
- Preflight request handling
- Credentials support
- Selective header and method allowance
- Configurable max-age for preflight caching

### Allowed Headers

- `Content-Type`
- `X-Requested-With`
- `X-CSRF-Token`

### Allowed Methods

- `GET`
- `POST` 
- `PUT`
- `DELETE`

## Referer Validation

### Protection Against

- Cross-site request forgery (CSRF)
- Clickjacking attacks
- Cross-origin data theft

### Configuration

- `STRICT_REFERER=true`: Requires referer header for state-changing operations
- `STRICT_REFERER=false`: Allows empty referer (for direct API calls)

### Exempted Endpoints

- `/health`
- `/api/health`
- `/metrics`
- `/api/metrics`

## Session Management

### Features

- Cryptographically secure session ID generation (256 bits entropy)
- Automatic session creation for new browsers
- Session validation and renewal
- Secure cookie attributes

### Cookie Security

- `HttpOnly`: Prevents JavaScript access
- `Secure`: Only sent over HTTPS
- `SameSite=Lax`: CSRF protection
- `Path=/api`: Restricts to API endpoints
- Long expiration time (1 year default) for permanent user identification

## Security Headers

### Implemented Headers

- `X-Content-Type-Options: nosniff` - Prevents MIME sniffing
- `X-Frame-Options: DENY` - Prevents clickjacking
- `X-XSS-Protection: 1; mode=block` - XSS protection for legacy browsers
- `Strict-Transport-Security` - Enforces HTTPS with configurable max-age
- `Content-Security-Policy` - Restrictive CSP policy
- `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer information
- `Permissions-Policy` - Restricts browser features
- `X-Permitted-Cross-Domain-Policies: none` - Blocks Adobe Flash policies
- `Cache-Control: no-cache, no-store, must-revalidate` - Prevents caching
- `X-DNS-Prefetch-Control: off` - Disables DNS prefetching

## Request Limits

### Size Limits

- Default: 1MB maximum request body size
- Configurable via `MAX_REQUEST_SIZE`
- Returns `413 Payload Too Large` when exceeded

### Timeout Limits

- Default: 30 seconds request timeout
- Configurable via `REQUEST_TIMEOUT`
- Prevents slow loris attacks

## Health Check Endpoint

The `/health` endpoint is available for monitoring and load balancer health checks:
- Exempt from rate limiting
- Exempt from referer validation
- Returns JSON with status and timestamp

## Logging and Monitoring

### Security Events Logged

- CORS violations (blocked origins)
- Rate limit violations
- Referer validation failures
- Invalid session attempts
- Suspicious request patterns

### Log Levels

- `INFO`: Normal security events (session creation, etc.)
- `WARN`: Security violations and blocked requests
- `DEBUG`: Detailed security flow information

### Recommended Production Logging

```bash
RUST_LOG=warn,word_game_backend=info
```

## Testing

Comprehensive integration tests are available in `src/security/tests.rs`:

- CORS validation
- Rate limiting behavior
- Referer validation
- Session management
- Security header presence
- Request size limits
- Health check exemptions

Run tests with:
```bash
cargo test security::tests
```

## Production Deployment Recommendations

### Environment Configuration

```bash
# Restrict to your actual domain
ALLOWED_ORIGINS=https://yourdomain.com

# Conservative rate limits
RATE_LIMIT_SESSION=5
RATE_LIMIT_READ=100
RATE_LIMIT_WRITE=20

# Strict security
STRICT_REFERER=true
HSTS_MAX_AGE=63072000

# Minimal logging
RUST_LOG=warn
```

### Load Balancer Configuration

Ensure your load balancer properly forwards client IPs:
- Set `X-Forwarded-For` header
- Set `X-Real-IP` header

### HTTPS Requirements

- Use TLS 1.2+ 
- Proper certificate chain
- HSTS preload list inclusion recommended

### Monitoring

Monitor the following metrics:
- Rate limit violations per IP
- CORS violations
- Failed referer validations
- Session creation rate
- Request timeout frequency

## Security Considerations

### Known Limitations

1. **Rate limiting is in-memory**: Not shared across multiple server instances
2. **Session storage**: Sessions are validated against database, but rate limits are per-instance
3. **IP spoofing**: Relies on proxy headers for IP detection

### Mitigation Strategies

1. **Distributed rate limiting**: Consider Redis-based rate limiting for multi-instance deployments
2. **Session clustering**: Use shared session storage for high availability
3. **Network-level protection**: Use CloudFlare or AWS WAF for additional protection
4. **Regular security audits**: Review logs and update security policies

### Future Enhancements

- [ ] Distributed rate limiting with Redis
- [ ] JWT token support for stateless authentication
- [ ] Geographic IP blocking
- [ ] Advanced bot detection
- [ ] WebAuthn support
- [ ] API key authentication for service-to-service calls