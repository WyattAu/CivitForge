# CDN Deployment Guide for CivitForge

This guide covers deploying CivitForge with a CDN for optimal performance, including CloudFlare, AWS CloudFront, cache invalidation, asset fingerprinting, and monitoring.

## CloudFlare Setup

### Initial Configuration

1. Add your domain to CloudFlare and update DNS nameservers at your registrar
2. In **SSL/TLS > Overview**: Set to "Full (strict)"
3. In **Speed > Optimization > Auto Minify**: Enable JS and CSS
4. In **Caching > Configuration**:

```
Browser Cache TTL: Respect Existing Headers
Caching Level: Standard
Always Online: ON
```

### Cache Rules

Create rules in **Caching > Cache Rules** (modern dashboard):

```
Rule 1: Static Assets (immutable cache)
  Expression: (http.request.uri.path wildcard "/assets/*")
  Cache: Cache, Edge TTL: 30 days, Browser TTL: 1 year

Rule 2: API (no cache)
  Expression: (http.request.uri.path wildcard "/api/*")
  Cache: Bypass

Rule 3: HTML (short cache)
  Expression: (http.request.uri.path wildcard "*.html")
  Cache: Cache, Edge TTL: 0s
```

### CloudFlare Workers (Advanced)

For fine-grained cache control, deploy a Worker:

```javascript
export default {
  async fetch(request) {
    const url = new URL(request.url);

    // Fingerprinted assets: immutable cache
    if (url.pathname.match(/-[a-f0-9]{64}\.(js|css|woff2|png|jpg|svg)$/)) {
      const response = await fetch(request);
      const newResponse = new Response(response.body, response);
      newResponse.headers.set('Cache-Control', 'public, max-age=31536000, immutable');
      return newResponse;
    }

    // API: pass through
    if (url.pathname.startsWith('/api/')) {
      return fetch(request);
    }

    // HTML: revalidate
    const response = await fetch(request);
    const newResponse = new Response(response.body, response);
    newResponse.headers.set('Cache-Control', 'public, max-age=0, must-revalidate');
    return newResponse;
  }
};
```

### Cache Purge

```bash
# Purge everything
curl -X DELETE "https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache" \
  -H "Authorization: Bearer {api_token}" \
  -H "Content-Type: application/json" \
  --data '{"purge_everything":true}'

# Purge specific URLs
curl -X DELETE "https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache" \
  -H "Authorization: Bearer {api_token}" \
  -H "Content-Type: application/json" \
  --data '{"files":["https://example.com/assets/old-hash.js"]}'
```

## AWS CloudFront Setup

### Terraform Distribution

```hcl
resource "aws_cloudfront_distribution" "civitforge" {
  enabled             = true
  default_root_object = "index.html"
  http_version        = "http2and3"

  origin {
    domain_name = "civitforge.example.com"
    origin_id   = "civitforge-primary"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  # Fingerprinted assets: aggressive caching
  ordered_cache_behavior {
    path_pattern           = "/assets/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "civitforge-primary"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true
    min_ttl                = 31536000
    default_ttl            = 31536000
    max_ttl                = 31536000

    forwarded_values {
      query_string = false
      cookies { forward = "none" }
    }
  }

  # API: no cache
  ordered_cache_behavior {
    path_pattern           = "/api/*"
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = []
    target_origin_id       = "civitforge-primary"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true
    min_ttl                = 0
    default_ttl            = 0
    max_ttl                = 0

    forwarded_values {
      query_string = true
      headers      = ["*"]
      cookies { forward = "all" }
    }
  }

  # HTML: short cache
  ordered_cache_behavior {
    path_pattern           = "*.html"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "civitforge-primary"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true
    min_ttl                = 0
    default_ttl            = 300
    max_ttl                = 3600

    forwarded_values {
      query_string = true
      headers      = ["Host"]
      cookies { forward = "none" }
    }
  }

  restrictions {
    geo_restriction { restriction_type = "none" }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }
}
```

### Cache Invalidation

```bash
# Invalidate specific files
aws cloudfront create-invalidation \
  --distribution-id E1234567890ABC \
  --paths "/index.html" "/api/*"

# Wildcard invalidation
aws cloudfront create-invalidation \
  --distribution-id E1234567890ABC \
  --paths "/*"
```

## Cache Invalidation Strategies

### Strategy 1: Content-Hash Fingerprinting (Recommended)

- CDN caches fingerprinted assets indefinitely
- New deploys create new filenames automatically
- Zero manual invalidation for static assets
- Old files expire naturally or are purged on deploy

```bash
deploy() {
  npm run build -- --fingerprint
  rsync -av dist/assets/ origin:/var/www/civitforge/assets/
  cp dist/index.html origin:/var/www/civitforge/
  purge_cdn_html
}
```

### Strategy 2: Version-Based Purge

```bash
VERSION=$(git rev-parse --short HEAD)
purge_cdn_by_version() {
  curl -X DELETE "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/purge_cache" \
    -H "Authorization: Bearer ${API_TOKEN}" \
    -d "{\"files\":[\"https://example.com/assets/*${VERSION}*\"]}"

  aws cloudfront create-invalidation \
    --distribution-id "${CF_DIST_ID}" \
    --paths "/assets/*${VERSION}*"
}
```

### Strategy 3: Time-Based TTL

| Asset Type | CDN TTL | Browser TTL | Rationale |
|------------|---------|-------------|-----------|
| Fingerprinted JS/CSS | 1 year | 1 year | Content changes via filename |
| Fingerprinted images | 1 year | 1 year | Content changes via filename |
| HTML | 5 min | 0 (revalidate) | Entry point, must stay fresh |
| API responses | No cache | No cache | Dynamic content |
| Service worker | No cache | No cache | Must always be latest |

## Asset Fingerprinting Deployment

### Build Configuration

CivitForge computes SHA-256 content hashes for static assets:

```
Original:     app.js
Fingerprinted: app-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2.js
```

### Deploy Script

```bash
#!/bin/bash
set -euo pipefail

ASSETS_DIR="dist/assets"
ORIGIN_DIR="/var/www/civitforge"

# Build with fingerprinting
cargo build --release --bin civit-ui --target wasm32-unknown-unknown

# Upload fingerprinted assets (long cache)
rsync -av --delete "$ASSETS_DIR/" "${ORIGIN_DIR}/assets/"

# Upload HTML (must always be fresh)
cp dist/index.html "${ORIGIN_DIR}/"
cp dist/favicon.ico "${ORIGIN_DIR}/" 2>/dev/null || true

# Purge CDN cache for HTML only (not fingerprinted assets)
purge_cdn_html

echo "Deploy complete."
```

### Verification

```bash
# Verify fingerprinted asset has correct headers
curl -sI "https://example.com/assets/app-a1b2c3d4.js" | grep -i cache-control
# Expected: Cache-Control: public, max-age=31536000, immutable

# Verify HTML has short cache
curl -sI "https://example.com/index.html" | grep -i cache-control
# Expected: Cache-Control: public, max-age=0, must-revalidate
```

## Performance Monitoring

### Key Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Cache Hit Ratio | > 95% | < 90% |
| TTFB (p95) | < 50ms | > 200ms |
| Origin Bandwidth Offload | > 80% | < 60% |
| Invalidation Latency | < 5 min | > 15 min |

### Prometheus Metrics

```yaml
scrape_configs:
  - job_name: 'civitforge-cdn'
    static_configs:
      - targets: ['civitforge:8080']
    metrics_path: '/api/v1/metrics'
```

### CloudFlare Analytics

```bash
curl "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/analytics/dashboard" \
  -H "Authorization: Bearer ${API_TOKEN}" \
  --data-urlbind 'since=-1440' \
  --data-urlbind 'until=0' \
  --data-urlbind 'metrics=cacheHitRatio,bytesSaved'
```

### CloudWatch Metrics (CloudFront)

```bash
# Cache hit rate
aws cloudwatch get-metric-statistics \
  --namespace AWS/CloudFront \
  --metric-name RequestCount \
  --dimensions Name=DistributionId,Value=E1234567890ABC \
  --start-time $(date -u -d '1 hour ago' +%FT%TZ) \
  --end-time $(date -u +%FT%TZ) \
  --period 300 \
  --statistics Sum
```

## Security Considerations

- **CSP headers**: Ensure CDN doesn't strip Content-Security-Policy
- **HSTS**: Enable at CDN level for HTTPS-only access
- **Origin shielding**: Use CDN origin shielding to reduce origin load
- **Rate limiting**: Configure CDN-level rate limiting for `/api/` paths
- **Bot protection**: Enable bot detection for non-browser traffic

## Performance Tuning Checklist

- [ ] HTTP/2 or HTTP/3 enabled at CDN edge
- [ ] Brotli compression enabled (prefer over gzip)
- [ ] TLS 1.3 enabled for faster handshakes
- [ ] Early hints (103) enabled for critical resources
- [ ] Origin keep-alive enabled
- [ ] Connection pooling configured for origin
- [ ] Edge location coverage matches user base
- [ ] Cache hit ratio > 95%
- [ ] Origin bandwidth < 20% of total
- [ ] TTFB < 50ms at 95th percentile
