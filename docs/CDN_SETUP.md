# CDN Setup Guide for CivitForge

This guide covers configuring a CDN in front of CivitForge for optimal static asset delivery and API performance.

## Overview

CivitForge's static asset system produces content-hashed filenames (e.g., `app-a1b2c3d4e5f6.js`) that can be safely cached indefinitely at the CDN edge. API responses are compressible and benefit from edge caching where safe.

## 1. Asset Fingerprinting

CivitForge computes SHA-256 content hashes for static assets. Fingerprinted files include a hex hash in the filename:

```
Original:     app.js
Fingerprinted: app-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2.js
```

This allows CDN edge servers to cache fingerprinted assets with `max-age=31536000, immutable` since the filename changes on every content update.

Non-fingerprinted files (HTML, API responses) receive shorter cache durations with `must-revalidate`.

## 2. CloudFlare Configuration

### Quick Setup

1. Add your domain to CloudFlare and update DNS nameservers
2. In **Speed > Optimization > Auto Minify**: Enable JS and CSS minification
3. In **Caching > Configuration**:

```text
Browser Cache TTL: Respect Existing Headers
Caching Level: Standard
Always Online: ON
```

### Page Rules (or Cache Rules in newer dashboard)

Create rules for the following URL patterns:

```text
Rule 1: Static Assets (immutable cache)
  URL Pattern: */assets/*
  Cache Level: Cache Everything
  Edge Cache TTL: 1 month
  Browser Cache TTL: 1 year
  
Rule 2: HTML Documents (short cache)
  URL Pattern: *.html
  Cache Level: Bypass (or Cache with 0s Edge TTL)
  
Rule 3: API Responses (no cache)
  URL Pattern: /api/*
  Cache Level: Bypass
```

### CloudFlare Workers (Optional)

For advanced cache control, deploy a Worker:

```javascript
export default {
  async fetch(request) {
    const url = new URL(request.url);
    
    // Fingerprinted assets: aggressive caching
    if (url.pathname.match(/-[a-f0-9]{64}\.(js|css|woff2|png|jpg|svg)$/)) {
      const response = await fetch(request);
      const newResponse = new Response(response.body, response);
      newResponse.headers.set('Cache-Control', 'public, max-age=31536000, immutable');
      return newResponse;
    }
    
    // API: pass through with no cache
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

### CloudFlare Page Rules vs Cache Rules

| Setting | Page Rules (Legacy) | Cache Rules (New) |
|---------|-------------------|-------------------|
| Static Assets | Cache Everything, 1 month | Cache, 30 days |
| API | Bypass | Custom: Bypass |
| HTML | Standard, 4h | Cache, 0 TTL |

### Purging Cache

```bash
# Purge everything
curl -X DELETE "https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache" \
  -H "Authorization: Bearer {api_token}" \
  -H "Content-Type: application/json" \
  --data '{"purge_everything":true}'

# Purge by URL (preferred for fingerprinted assets)
curl -X DELETE "https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache" \
  -H "Authorization: Bearer {api_token}" \
  -H "Content-Type: application/json" \
  --data '{"files":["https://example.com/assets/old-hash.js"]}'
```

## 3. AWS CloudFront Configuration

### Distribution Setup

```hcl
# Terraform / CloudFormation example
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

  # Default behavior: proxy everything
  default_cache_behavior {
    allowed_methods        = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods         = ["GET", "HEAD", "OPTIONS"]
    target_origin_id       = "civitforge-primary"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true
    min_ttl                = 0
    default_ttl            = 86400
    max_ttl                = 31536000
    
    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Accept-Encoding", "Host"]
      
      cookies {
        forward = "none"
      }
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
      cookies {
        forward = "none"
      }
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
      
      cookies {
        forward = "all"
      }
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
      
      cookies {
        forward = "none"
      }
    }
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }
}
```

### CloudFront Cache Invalidation

```bash
# Invalidate specific files (use for non-fingerprinted assets)
aws cloudfront create-invalidation \
  --distribution-id E1234567890ABC \
  --paths "/index.html" "/api/*"

# Wildcard invalidation
aws cloudfront create-invalidation \
  --distribution-id E1234567890ABC \
  --paths "/*"
```

### Lambda@Edge for Custom Headers (Optional)

```javascript
// Lambda@Edge origin response handler
exports.handler = async (event) => {
  const response = event.Records[0].cf.response;
  const headers = response.headers;
  const uri = event.Records[0].cf.request.uri;
  
  // Add cache headers for fingerprinted assets
  if (uri.match(/-[a-f0-9]{64}\./)) {
    headers['cache-control'] = [{
      key: 'Cache-Control',
      value: 'public, max-age=31536000, immutable'
    }];
  }
  
  return response;
};
```

## 4. Nginx CDN Configuration

### Basic CDN Setup

```nginx
upstream civitforge_backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

# Proxy cache zone (10GB, 60 minute inactive purge)
proxy_cache_path /var/cache/civitforge 
    levels=1:2 
    keys_zone=civitforge_cache:100m 
    max_size=10g 
    inactive=60m 
    use_temp_path=off;

server {
    listen 443 ssl http2;
    server_name civitforge.example.com;

    ssl_certificate /etc/ssl/certs/civitforge.pem;
    ssl_certificate_key /etc/ssl/private/civitforge.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Gzip/Brotli compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml image/svg+xml;
    
    brotli on;
    brotli_comp_level 6;
    brotli_types text/plain text/css application/json application/javascript text/xml application/xml image/svg+xml;

    # Fingerprinted static assets
    location ~* ^/assets/.*-[a-f0-9]{64}\.(js|css|woff2|woff|ttf|eot|png|jpg|jpeg|gif|webp|avif|svg|ico|map)$ {
        proxy_pass http://civitforge_backend;
        proxy_cache civitforge_cache;
        proxy_cache_valid 200 365d;
        proxy_cache_use_stale error timeout updating http_500 http_502 http_503;
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header X-Cache-Status $upstream_cache_status;
        expires 365d;
    }

    # Non-fingerprinted static assets
    location /assets/ {
        proxy_pass http://civitforge_backend;
        proxy_cache civitforge_cache;
        proxy_cache_valid 200 1h;
        add_header Cache-Control "public, max-age=3600, must-revalidate";
        add_header X-Cache-Status $upstream_cache_status;
    }

    # HTML files
    location ~* \.html$ {
        proxy_pass http://civitforge_backend;
        proxy_cache civitforge_cache;
        proxy_cache_valid 200 5m;
        add_header Cache-Control "public, max-age=0, must-revalidate";
        add_header X-Cache-Status $upstream_cache_status;
    }

    # API: no cache, pass through
    location /api/ {
        proxy_pass http://civitforge_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        add_header Cache-Control "no-store, no-cache, must-revalidate";
    }

    # Git smart HTTP
    location ~ ^/(.+)/(.+)/(info/refs|git-upload-pack|git-receive-pack)$ {
        proxy_pass http://civitforge_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        client_max_body_size 10g;
    }

    # Default: pass through
    location / {
        proxy_pass http://civitforge_backend;
        proxy_cache civitforge_cache;
        proxy_cache_valid 200 5m;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        add_header X-Cache-Status $upstream_cache_status;
    }
}
```

### Cache Invalidation Script

```bash
#!/bin/bash
# invalidate-nginx-cache.sh
CACHE_DIR="/var/cache/civitforge"
PATTERN="${1:-*}"

echo "Clearing cache for pattern: $PATTERN"
find "$CACHE_DIR" -name "$PATTERN" -delete
echo "Cache invalidated."
```

## 5. Cache Invalidation Strategies

### Strategy 1: Content-Hash Fingerprinting (Recommended)

- **Mechanism**: CDN caches fingerprinted assets indefinitely; new deploys create new filenames
- **Invalidation**: Automatic — old files expire naturally or are purged on deploy
- **Pros**: Zero manual invalidation for static assets, perfect cache hit rates
- **Cons**: Requires build tooling to fingerprint; old files remain until purged

```bash
# Deploy script example
deploy() {
  # Build with fingerprinting
  npm run build -- --fingerprint
  
  # Upload new assets to origin
  rsync -av dist/assets/ origin:/var/www/civitforge/assets/
  
  # Update index.html to reference new hashes
  cp dist/index.html origin:/var/www/civitforge/
  
  # Purge CDN cache for HTML only
  purge_cdn_html
  
  # Optionally purge old fingerprinted assets
  purge_old_fingerprints
}
```

### Strategy 2: Version-Based Purge

```bash
# Purge by version tag
VERSION=$(git rev-parse --short HEAD)
purge_cdn_by_version() {
  # CloudFlare: purge specific URLs
  curl -X DELETE "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/purge_cache" \
    -H "Authorization: Bearer ${API_TOKEN}" \
    -d "{\"files\":[\"https://example.com/assets/*${VERSION}*\"]}"
  
  # CloudFront: wildcard invalidation
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

## 6. Monitoring CDN Performance

### Key Metrics to Track

- **Cache Hit Ratio**: Target > 95% for static assets
- **Time to First Byte (TTFB)**: Target < 50ms at edge
- **Origin bandwidth reduction**: Target > 80% offload
- **Cache invalidation latency**: Track how quickly changes propagate

### Prometheus Metrics

CivitForge exposes `/api/v1/metrics` in Prometheus format:

```yaml
# Prometheus scrape config
scrape_configs:
  - job_name: 'civitforge'
    static_configs:
      - targets: ['civitforge:8080']
    metrics_path: '/api/v1/metrics'
```

### CloudFlare Analytics

```bash
# Fetch cache analytics via API
curl "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/analytics/dashboard" \
  -H "Authorization: Bearer ${API_TOKEN}" \
  --data-urlbind 'since=-1440' \
  --data-urlbind 'until=0' \
  --data-urlbind 'metrics=cacheHitRatio,bytesSaved'
```

## 7. Security Considerations

- **CSP headers**: CivitForge sets Content-Security-Policy; ensure CDN doesn't strip them
- **HSTS**: Enable HSTS at CDN level for HTTPS-only access
- **Origin shielding**: Use CDN origin shielding to reduce origin load
- **Rate limiting**: Configure CDN-level rate limiting for `/api/` paths
- **Bot protection**: Enable bot detection for non-browser traffic

## 8. Performance Tuning Checklist

- [ ] HTTP/2 or HTTP/3 enabled at CDN edge
- [ ] Brotli compression enabled at CDN (prefer over gzip)
- [ ] TLS 1.3 enabled for faster handshakes
- [ ] Early hints (103) enabled for critical resources
- [ ] Origin keep-alive enabled
- [ ] Connection pooling configured for origin
- [ ] Edge location coverage matches user base
- [ ] Cache hit ratio > 95%
- [ ] Origin bandwidth < 20% of total
- [ ] TTFB < 50ms at 95th percentile
