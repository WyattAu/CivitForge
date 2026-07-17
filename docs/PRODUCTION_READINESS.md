# CivitForge Production Readiness Checklist

This document covers infrastructure requirements, configuration, security, monitoring, backups, scaling, and troubleshooting for running CivitForge in production.

## Infrastructure Requirements

### Minimum (Small Team / <500 Repos)

| Resource | Requirement |
|----------|-------------|
| CPU | 4 cores |
| RAM | 8 GB |
| Disk | 100 GB SSD |
| Network | 100 Mbps |

### Recommended (Medium / 500-5000 Repos)

| Resource | Requirement |
|----------|-------------|
| CPU | 8 cores |
| RAM | 16 GB |
| Disk | 500 GB NVMe SSD |
| Network | 1 Gbps |

### Large Scale (5000+ Repos)

| Resource | Requirement |
|----------|-------------|
| CPU | 16+ cores (or horizontal scaling) |
| RAM | 32 GB+ |
| Disk | 2 TB+ NVMe SSD (or object storage) |
| Network | 10 Gbps |

### External Services

| Service | Version | Purpose |
|---------|---------|---------|
| PostgreSQL | 17+ | Primary database |
| Redis | 7+ | Caching, sessions, pub/sub |
| Object Storage | S3-compatible | Large file storage (optional) |

## Configuration Checklist

### Environment Variables

```bash
# Required
DATABASE_URL=postgres://civit:PASSWORD@host:5432/civit
JWT_SECRET=<random-64-char-hex-string>

# Recommended
CIVIT_HOST=0.0.0.0
CIVIT_PORT=8080
REDIS_URL=redis://:PASSWORD@host:6379
JWT_EXPIRY_HOURS=24
CIVIT_STORAGE_PATH=/var/lib/civit/repos
RUST_LOG=civit_core=info,tower_http=info

# Optional
FEDERATION_ENABLED=false
FEDERATION_INSTANCE_ID=<unique-id>
FEDERATION_INSTANCE_DOMAIN=<your-domain>
```

### Generate Secrets

```bash
# JWT secret (32 bytes hex)
openssl rand -hex 32

# Database password
openssl rand -base64 32

# Redis password
openssl rand -base64 24
```

## Security Checklist

### Authentication & Authorization

- [ ] JWT secret is randomly generated (>= 32 bytes)
- [ ] JWT expiry is set appropriately (default: 24h)
- [ ] Token refresh mechanism is in place
- [ ] Role-based access control is configured

### Network Security

- [ ] TLS termination at load balancer or reverse proxy
- [ ] Internal services communicate over private network
- [ ] SSH gateway (port 2222) is restricted or behind VPN
- [ ] Rate limiting is configured

### Data Security

- [ ] Database connections use TLS (`?sslmode=require`)
- [ ] Redis connections use TLS in production
- [ ] Encryption at rest for storage volumes
- [ ] Secrets are not committed to version control

### System Hardening

- [ ] Non-root user runs CivitForge processes
- [ ] Firewall rules restrict inbound traffic
- [ ] Fail2ban or equivalent for SSH protection
- [ ] Regular security updates applied

## Monitoring Setup

### Health Checks

```bash
# Quick health check
scripts/monitoring/health_check.sh --compact

# Full JSON report
scripts/monitoring/health_check.sh

# Continuous monitoring
scripts/monitoring/health_check.sh --watch 10
```

### Key Metrics to Monitor

| Metric | Warning | Critical |
|--------|---------|----------|
| API response p95 | >500ms | >1000ms |
| Error rate | >1% | >5% |
| DB connection pool usage | >80% | >95% |
| Redis memory usage | >70% | >85% |
| Disk usage | >75% | >90% |
| Memory usage | >80% | >95% |

### Endpoints to Monitor

| Endpoint | Purpose | Expected |
|----------|---------|----------|
| `GET /healthz` | Liveness probe | 200 OK |
| `GET /ready` | Readiness probe | 200 OK |
| `GET /api/v1/health` | Service health | 200 OK |

### Recommended Tooling

- **Prometheus + Grafana**: Metrics collection and dashboards
- **Alertmanager**: PagerDuty/Slack alerts
- **Loki**: Log aggregation
- **Tempo**: Distributed tracing

## Backup Procedures

### Database Backups

```bash
# Automated daily backup (add to cron)
pg_dump -Fc -U civit civit > /backups/civit-$(date +%Y%m%d).dump

# Restore
pg_restore -U civit -d civit /backups/civit-20260717.dump
```

### Storage Backups

```bash
# Sync repos to backup location
rsync -avz /var/lib/civit/repos/ backup-host:/backups/repos/

# For S3-compatible storage
aws s3 sync /var/lib/civit/repos/ s3://your-backup-bucket/repos/
```

### Backup Schedule

| Data | Frequency | Retention |
|------|-----------|-----------|
| PostgreSQL | Daily | 30 days |
| Repositories | Daily incremental | 30 days |
| Configuration | On change | Indefinite |

### Verify Backups

```bash
# Test restore to staging
scripts/disaster_recovery_test.sh

# Verify dump integrity
pg_restore -l /backups/civit-latest.dump > /dev/null
```

## Scaling Guidelines

### Vertical Scaling

When to scale up:
- CPU consistently >70%
- Memory usage >80%
- Response times increasing under load

### Horizontal Scaling

For multi-instance deployment:

1. **Load balancer**: Distribute traffic across CivitForge instances
2. **Shared storage**: Use NFS/S3 for repository storage
3. **Session store**: Redis handles session sharing automatically
4. **Database**: Consider read replicas for read-heavy workloads

### Component Scaling

| Component | Approach |
|-----------|----------|
| CivitForge API | Horizontal (load balancer) |
| Brain (search) | Vertical or horizontal |
| Runner (CI) | Horizontal (add runners) |
| PostgreSQL | Read replicas + connection pooling |
| Redis | Cluster mode for high throughput |

## Troubleshooting Guide

### Common Issues

#### High Response Times

```bash
# Check system resources
scripts/monitoring/health_check.sh

# Check database connections
pg_isready -h 127.0.0.1 -p 5432
SELECT count(*) FROM pg_stat_activity;

# Check Redis
redis-cli info memory
```

#### Database Connection Errors

```bash
# Verify connectivity
psql "$DATABASE_URL" -c "SELECT 1;"

# Check connection pool
docker compose exec postgres psql -U civit -c \
  "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# Kill idle connections
SELECT pg_terminate_backend(pid) FROM pg_stat_activity
  WHERE state = 'idle' AND query_start < now() - interval '1 hour';
```

#### Redis Connection Errors

```bash
# Test connection
redis-cli -a PASSWORD ping

# Check memory
redis-cli -a PASSWORD info memory

# Check connected clients
redis-cli -a PASSWORD info clients
```

#### Disk Space Issues

```bash
# Find large files
du -sh /var/lib/civit/repos/* | sort -rh | head -20

# Clean old build artifacts
docker system prune -af
```

### Disaster Recovery

```bash
# Run full DR test
scripts/disaster_recovery_test.sh

# Skip backup restore test
scripts/disaster_recovery_test.sh --skip-backup

# Dry run
scripts/disaster_recovery_test.sh --dry-run
```

### Performance Debugging

```bash
# Load test
k6 run scripts/loadtest/k6_api_loadtest.js

# Check for regressions
scripts/check_perf_regression.sh

# CPU profiling
perf record -g -p $(pgrep civit-core)
perf report
```

## Deployment Verification

After deploying to production, verify:

1. [ ] Health checks pass
2. [ ] Authentication works
3. [ ] Repository CRUD operations work
4. [ ] Git HTTP/SSH access works
5. [ ] Search indexing works
6. [ ] Pipeline execution works
7. [ ] Federation (if enabled) connects
8. [ ] Backup verification succeeds
9. [ ] Monitoring dashboards show data
10. [ ] Alerting rules are active
