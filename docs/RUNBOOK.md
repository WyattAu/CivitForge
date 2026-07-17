# CivitForge Runbook

Operational procedures for CivitForge production deployments.

## Service Startup/Shutdown Procedures

### Docker Compose Startup

```bash
# Start all services
docker compose up -d

# Start with logs
docker compose up -d && docker compose logs -f

# Verify health
curl -sf http://localhost:9091/healthz || echo "FAILED"
```

### Docker Compose Shutdown

```bash
# Graceful shutdown (sends SIGTERM, waits 30s)
docker compose down

# Force shutdown (sends SIGKILL immediately)
docker compose down --remove-orphans

# Shutdown and remove volumes (DESTRUCTIVE)
docker compose down -v
```

### Kubernetes Startup

```bash
# Verify pods are running
kubectl get pods -n civitforge -w

# Check rollout status
kubectl rollout status deployment/civitforge-api -n civitforge
kubectl rollout status deployment/civitforge-runner -n civitforge
kubectl rollout status deployment/civitforge-vfs -n civitforge

# Check all services healthy
kubectl get pods -n civitforge -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}'
```

### Kubernetes Shutdown

```bash
# Scale down gracefully
kubectl scale deployment civitforge-api --replicas=0 -n civitforge
kubectl scale deployment civitforge-runner --replicas=0 -n civitforge
kubectl scale deployment civitforge-brain --replicas=0 -n civitforge
kubectl scale deployment civitforge-vfs --replicas=0 -n civitforge

# Or uninstall entirely
helm uninstall civitforge -n civitforge

# Remove PVCs manually if needed
kubectl delete pvc -l app.kubernetes.io/instance=civitforge -n civitforge
```

### Source Build Startup

```bash
# Build
cargo build --release --workspace

# Start
./target/release/civit-core &

# Verify
curl http://localhost:8080/healthz
```

## Database Migration Procedures

### Automatic Migrations

CivitForge runs migrations automatically on startup via sqlx. No manual action required for normal upgrades.

### Manual Migration Check

```sql
-- Check current migration state
SELECT * FROM schema_migrations ORDER BY version;

-- Count applied migrations
SELECT COUNT(*) FROM schema_migrations;
```

### Manual Migration (If Auto-Migrate Disabled)

```bash
# Apply all pending migrations
cargo sqlx migrate run --source crates/civit-db/src/migrations

# Check migration status
cargo sqlx migrate info --source crates/civit-db/src/migrations
```

### Rollback

```bash
# Rollback last migration
cargo sqlx migrate revert --source crates/civit-db/src/migrations

# Verify rollback
SELECT * FROM schema_migrations ORDER BY version DESC LIMIT 5;
```

### Migration Troubleshooting

**Migration fails with permission error:**

```sql
-- Ensure user has required privileges
GRANT ALL PRIVILEGES ON DATABASE civitforge TO civitforge;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO civitforge;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO civitforge;
ALTER DATABASE civitforge OWNER TO civitforge;
```

**Migration fails on lock contention:**

```sql
-- Check for long-running queries
SELECT pid, state, query, age(clock_timestamp(), query_start) AS age
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY age DESC;

-- Terminate blocking queries if safe
SELECT pg_terminate_backend(<pid>);
```

**Rollback migration files:**

Rollback SQL files are in `crates/civit-db/src/migrations/down/`. Each numbered migration has a corresponding rollback.

## Backup/Restore Procedures

### Database Backup

```bash
# Full database dump
pg_dump -Fc -h localhost -p 5432 -U civit -d civitforge \
  > civitforge-db-$(date +%Y%m%d-%H%M%S).dump

# Schema only
pg_dump -s -h localhost -p 5432 -U civit -d civitforge \
  > civitforge-schema-$(date +%Y%m%d).sql

# Data only (specific tables)
pg_dump -a -h localhost -p 5432 -U civit -d civitforge \
  -t users -t repositories -t pipelines \
  > civitforge-data-$(date +%Y%m%d).sql
```

### Database Restore

```bash
# Full restore
pg_restore -Fc -h localhost -p 5432 -U civit -d civitforge \
  civitforge-db-YYYYMMDD-HHMMSS.dump

# Schema only
psql -h localhost -p 5432 -U civit -d civitforge \
  < civitforge-schema-YYYYMMDD.sql
```

### Git Repository Backup

```bash
# Backup all repositories
tar czf repos-backup-$(date +%Y%m%d).tar.gz /var/lib/civit/repos/

# Backup specific repository
tar czf repo-backup-$(date +%Y%m%d).tar.gz \
  /var/lib/civit/repos/my-org/my-repo.git/

# Mirror specific repo (for remote backup)
git mirror /var/lib/civit/repos/my-org/my-repo.git /backup/my-repo.git
```

### Git Repository Restore

```bash
# Restore all repositories
tar xzf repos-backup-YYYYMMDD.tar.gz -C /

# Restore specific repository
tar xzf repo-backup-YYYYMMDD.tar.gz -C /var/lib/civit/repos/

# Fix permissions after restore
chown -R 65532:65532 /var/lib/civit/repos/
chmod -R 750 /var/lib/civit/repos/
```

### Kubernetes Backup

```bash
# Backup PVC data
kubectl exec -it <vfs-pod> -n civitforge -- \
  tar czf /tmp/backup.tar.gz /data/

kubectl cp civitforge/<vfs-pod>:/tmp/backup.tar.gz ./vfs-backup-$(date +%Y%m%d).tar.gz

# Backup Helm values
helm get values civitforge -n civitforge > helm-values-backup.yaml
```

### Automated Backup Script

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/backup/civitforge/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Database
pg_dump -Fc -h localhost -p 5432 -U civit -d civitforge \
  > "$BACKUP_DIR/civitforge.dump"

# Repositories
tar czf "$BACKUP_DIR/repos.tar.gz" /var/lib/civit/repos/

# Cleanup old backups (keep 30 days)
find /backup/civitforge -maxdepth 1 -type d -mtime +30 -exec rm -rf {} +

echo "Backup complete: $BACKUP_DIR"
```

## Scaling Procedures

### Horizontal Scaling (Kubernetes)

```bash
# Scale API
kubectl scale deployment civitforge-api --replicas=5 -n civitforge

# Scale Runner
kubectl scale deployment civitforge-runner --replicas=4 -n civitforge

# Scale VFS
kubectl scale deployment civitforge-vfs --replicas=3 -n civitforge

# Verify scaling
kubectl get deployments -n civitforge
```

### Vertical Scaling (Kubernetes)

```bash
# Update resource limits
kubectl patch deployment civitforge-api -n civitforge -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"limits":{"cpu":"2","memory":"1Gi"}}}]}}}}'

# Or update via Helm
helm upgrade civitforge deploy/helm/civitforge \
  --namespace civitforge \
  --set api.resources.limits.cpu=2 \
  --set api.resources.limits.memory=1Gi
```

### HPA Configuration

```bash
# Check HPA status
kubectl get hpa -n civitforge

# Describe HPA
kubectl describe hpa civitforge-api -n civitforge

# Disable HPA (use fixed replicas)
kubectl patch hpa civitforge-api -n civitforge -p \
  '{"spec":{"minReplicas":5,"maxReplicas":5}}'
```

### Docker Compose Scaling

```bash
# Scale API (Docker Compose v2)
docker compose up -d --scale api=3

# Note: Port conflicts may occur; adjust port mappings
```

## Incident Response Procedures

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| P1 | Service completely down | Immediate | All APIs unreachable, database down |
| P2 | Major feature unavailable | 30 minutes | CI/CD pipelines not running, federation broken |
| P3 | Minor feature degraded | 2 hours | Slow API responses, intermittent errors |
| P4 | Cosmetic/minor issue | Next business day | UI bugs, minor logging issues |

### P1: Service Down

1. **Acknowledge** the alert
2. **Check** service status:
   ```bash
   kubectl get pods -n civitforge
   kubectl logs -l app.kubernetes.io/component=api -n civitforge --tail=50
   ```
3. **Triage** root cause:
   ```bash
   # Check resource exhaustion
   kubectl top pods -n civitforge

   # Check events
   kubectl get events -n civitforge --sort-by='.lastTimestamp' | tail -20

   # Check node health
   kubectl get nodes
   kubectl describe node <node-name>
   ```
4. **Mitigate**:
   ```bash
   # Restart affected component
   kubectl rollout restart deployment/civitforge-api -n civitforge

   # Scale up if needed
   kubectl scale deployment civitforge-api --replicas=6 -n civitforge
   ```
5. **Verify** recovery:
   ```bash
   curl -sf http://localhost:9091/healthz
   kubectl get pods -n civitforge
   ```
6. **Post-mortem**: Document root cause and prevention

### P2: Major Feature Unavailable

1. **Identify** affected component
2. **Check** component logs
3. **Attempt** restart
4. **Escalate** to P1 if restart fails
5. **Document** in incident channel

### P3: Performance Degradation

1. **Monitor** metrics for 5 minutes
2. **Check** resource utilization
3. **Identify** bottleneck
4. **Scale** if needed
5. **Monitor** improvement

## Common Troubleshooting

### Database Connection Refused

```bash
# Check PostgreSQL is running
kubectl get pods -n civitforge -l app.kubernetes.io/name=postgresql
kubectl logs -n civitforge -l app.kubernetes.io/name=postgresql --tail=20

# Test connectivity
kubectl exec -it <api-pod> -n civitforge -- \
  pg_isready -h postgresql -p 5432 -U civitforge

# Check connection pool
kubectl exec -it <api-pod> -n civitforge -- \
  curl -s http://localhost:8080/healthz
```

### Redis Connection Refused

```bash
# Check Redis is running
kubectl get pods -n civitforge -l app.kubernetes.io/name=redis
kubectl logs -n civitforge -l app.kubernetes.io/name=redis --tail=20

# Test connectivity
kubectl exec -it <api-pod> -n civitforge -- \
  redis-cli -h redis ping
```

### JWT Validation Errors

```bash
# Verify JWT secret exists
kubectl get secret -n civitforge -l app.kubernetes.io/name=civitforge

# Check secret content (base64 decode)
kubectl get secret civitforge-secrets -n civitforge -o jsonpath='{.data.jwt-secret}' | base64 -d

# Regenerate JWT secret
NEW_SECRET=$(openssl rand -base64 32)
kubectl patch secret civitforge-secrets -n civitforge -p \
  "{\"data\":{\"jwt-secret\":\"$(echo -n $NEW_SECRET | base64)\"}}"
```

### LDAP Authentication Failures

```bash
# Test LDAP connectivity
kubectl exec -it <api-pod> -n civitforge -- \
  ldapsearch -x -H ldap://ldap.example.com:389 \
  -b "ou=users,dc=example,dc=com" \
  -D "cn=admin,dc=example,dc=com" -w "password"

# Check LDAP config
kubectl get configmap civitforge-config -n civitforge -o yaml
```

### Port 8080 Already in Use

```bash
# Find process using port
lsof -i :8080

# Or in Kubernetes, check service conflicts
kubectl get svc -n civitforge -o wide
```

### Pod CrashLoopBackOff

```bash
# Get pod logs
kubectl logs <pod-name> -n civitforge --previous

# Check events
kubectl describe pod <pod-name> -n civitforge

# Common causes:
# - Invalid environment variables
# - Missing secrets
# - Database unreachable
# - Insufficient resources

# Debug
kubectl exec -it <pod-name> -n civitforge -- /bin/sh
```

### Git SSH Not Working

```bash
# Check SSH service
kubectl get svc -n civitforge -l app.kubernetes.io/component=api

# Test SSH connectivity
ssh -p 2222 git@forge.example.com

# Check host key
kubectl exec -it <api-pod> -n civitforge -- \
  cat /etc/ssh/ssh_host_ed25519_key.pub

# Verify SSH config on client
cat ~/.ssh/config | grep -A5 forge
```

### Memory Pressure

```bash
# Check pod memory usage
kubectl top pods -n civitforge --sort-by=memory

# Check node memory
kubectl top nodes

# Identify OOM kills
kubectl get events -n civitforge --field-selector reason=OOMKilling

# Increase memory limit
kubectl patch deployment civitforge-api -n civitforge -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"api","resources":{"limits":{"memory":"2Gi"}}}]}}}}'
```

### Disk Space Issues

```bash
# Check VFS storage
kubectl exec -it <vfs-pod> -n civitforge -- df -h /data

# Check PostgreSQL storage
kubectl exec -it <postgres-pod> -n civitforge -- df -h /var/lib/postgresql/data

# Clean up old git repos (careful!)
kubectl exec -it <vfs-pod> -n civitforge -- \
  find /data -name "*.git" -mtime +90 -type d | head -20

# Expand PVC
kubectl patch pvc civitforge-vfs -n civitforge -p \
  '{"spec":{"resources":{"requests":{"storage":"100Gi"}}}}'
```

### High CPU Usage

```bash
# Profile CPU usage
kubectl top pods -n civitforge --sort-by=cpu

# Check for infinite loops or busy-wait
kubectl exec -it <pod-name> -n civitforge -- \
  curl -s http://localhost:8080/debug/pprof/profile?seconds=10 > cpu.prof

# Analyze with pprof
go tool pprof cpu.prof
```

### Network Connectivity Issues

```bash
# Check NetworkPolicy
kubectl get networkpolicies -n civitforge

# Test inter-service connectivity
kubectl exec -it <api-pod> -n civitforge -- \
  curl -s http://civitforge-vfs:8083/healthz

# Check DNS resolution
kubectl exec -it <api-pod> -n civitforge -- \
  nslookup civitforge-vfs.civitforge.svc.cluster.local
```

## Log Analysis

### Common Log Patterns

```bash
# Error rate spike
kubectl logs -l app.kubernetes.io/component=api -n civitforge --since=1h | \
  grep -c "level=error"

# Authentication failures
kubectl logs -l app.kubernetes.io/component=api -n civitforge --since=1h | \
  grep "authentication" | grep "failed"

# Slow requests
kubectl logs -l app.kubernetes.io/component=api -n civitforge --since=1h | \
  grep "duration" | awk -F'duration=' '{if($2 > 2.0) print}'

# Pipeline failures
kubectl logs -l app.kubernetes.io/component=runner -n civitforge --since=1h | \
  grep "pipeline" | grep -i "fail\|error"
```

### Log Aggregation Queries

**Fluent Bit / Elasticsearch:**

```
# All errors in last hour
namespace:civitforge AND level:error AND @timestamp:[now-1h TO now]

# API errors
namespace:civitforge AND component:api AND level:error

# Database connection errors
namespace:civitforge AND message:"connection refused"
```

**Loki / LogQL:**

```
# All errors
{namespace="civitforge"} |= "level=error"

# API errors with context
{namespace="civitforge", component="api"} | json | level="error"

# Slow requests (> 2s)
{namespace="civitforge"} | json | duration > 2s

# Specific user actions
{namespace="civitforge"} | json | user_id="user-123"
```

## Health Check Reference

| Endpoint | Method | Response | Purpose |
|----------|--------|----------|---------|
| `GET /healthz` | HTTP | `OK` (200) | Liveness probe |
| `GET /ready` | HTTP | `OK` (200) | Readiness probe |
| `GET /api/v1/health` | HTTP | JSON | Detailed health |
| `GET /metrics` | HTTP | Prometheus | Metrics endpoint |

### Manual Health Check

```bash
# Quick check
curl -sf http://localhost:9091/healthz

# Detailed check
curl -s http://localhost:9091/api/v1/health | jq .

# Check all services
for svc in api runner brain vfs; do
  echo -n "$svc: "
  curl -sf http://localhost:9091/healthz && echo "OK" || echo "FAILED"
done
```

## Emergency Contacts

| Role | Contact | Escalation |
|------|---------|------------|
| On-call Engineer | PagerDuty rotation | P1: immediate |
| Platform Lead | Slack @platform-lead | P2: 30 min |
| Database Admin | Slack @dba | Database issues |
| Security Team | Slack @security | Security incidents |
