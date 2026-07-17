# CivitForge Helm Chart

Helm chart for deploying CivitForge federated forge on Kubernetes.

## Prerequisites

- Kubernetes 1.30+
- Helm 3.14+
- Ingress controller (nginx recommended)
- cert-manager (for TLS)
- PostgreSQL 17+ (standalone or Bitnami subchart)
- Redis 7+ (standalone or Bitnami subchart)

## Installation

### Quick Start

```bash
helm repo add civitforge https://charts.civitforge.io
helm repo update

helm install civitforge civitforge/civitforge \
  --namespace civitforge \
  --create-namespace \
  --set secrets.jwtSecret="$(openssl rand -base64 32)" \
  --set secrets.databaseUrl="postgres://civit:password@postgres:5432/civitforge" \
  --set secrets.redisPassword="$(openssl rand -base64 24)"
```

### Production Install

```bash
helm install civitforge civitforge/civitforge \
  --namespace civitforge \
  --create-namespace \
  -f values-production.yaml \
  --set secrets.jwtSecret="$(openssl rand -base64 32)" \
  --set secrets.databaseUrl="postgres://civit:PASSWORD@your-pg-host:5432/civitforge?sslmode=require" \
  --set secrets.redisPassword="$(openssl rand -base64 24)"
```

### Minimal (External DB/Redis)

```bash
helm install civitforge civitforge/civitforge \
  --namespace civitforge \
  --create-namespace \
  --set postgres.enabled=false \
  --set redis.enabled=false \
  --set secrets.jwtSecret="$(openssl rand -base64 32)" \
  --set secrets.databaseUrl="postgres://civit:PASSWORD@external-pg:5432/civitforge" \
  --set secrets.redisPassword="REDIS_PASSWORD"
```

## Architecture

The chart deploys four core services:

| Service | Port | Purpose |
|---------|------|---------|
| api | 8080 | HTTP/WebSocket API server |
| runner | 8081 | CI/CD pipeline execution engine |
| brain | 8082 | AI/ML services |
| vfs | 8083 | Virtual file system |

Plus infrastructure:

| Component | Purpose |
|-----------|---------|
| PostgreSQL | Primary database (Bitnami subchart or external) |
| Redis | Session cache, edge cache, pub/sub |
| NetworkPolicy | Zero-trust inter-service communication |
| PDB | Pod disruption budgets for safe rollouts |
| HPA | Horizontal pod autoscaling |

## Configuration

### Core Services

| Parameter | Description | Default |
|-----------|-------------|---------|
| `api.enabled` | Deploy API service | `true` |
| `api.replicas` | Number of API replicas | `3` |
| `api.image.repository` | API image | `civitforge/api` |
| `api.image.tag` | API image tag | `2.2.0` |
| `api.resources.requests.cpu` | API CPU request | `250m` |
| `api.resources.requests.memory` | API memory request | `256Mi` |
| `api.resources.limits.cpu` | API CPU limit | `1` |
| `api.resources.limits.memory` | API memory limit | `512Mi` |
| `runner.enabled` | Deploy runner service | `true` |
| `runner.replicas` | Number of runner replicas | `2` |
| `runner.resources.limits.cpu` | Runner CPU limit | `2` |
| `runner.resources.limits.memory` | Runner memory limit | `1Gi` |
| `runner.sandboxRuntime` | Sandbox runtime (podman/docker) | `podman` |
| `brain.enabled` | Deploy brain service | `true` |
| `brain.replicas` | Number of brain replicas | `1` |
| `vfs.enabled` | Deploy VFS service | `true` |
| `vfs.replicas` | Number of VFS replicas | `2` |
| `vfs.storage.size` | VFS PVC size | `50Gi` |

### Ingress

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class | `nginx` |
| `ingress.annotations` | Ingress annotations | nginx + cert-manager |
| `ingress.hosts[0].host` | Hostname | `forge.example.com` |
| `ingress.hosts[0].paths` | Path rules | `/`, `/ws`, `/vfs`, `/runner` |
| `ingress.tls[0].secretName` | TLS secret | `civitforge-tls` |

#### Ingress Examples

**Path-based routing (default):**

```yaml
ingress:
  hosts:
    - host: forge.example.com
      paths:
        - path: /
          pathType: Prefix
          service: api
          port: 8080
        - path: /ws
          pathType: Prefix
          service: api
          port: 8080
        - path: /vfs
          pathType: Prefix
          service: vfs
          port: 8083
        - path: /runner
          pathType: Prefix
          service: runner
          port: 8081
```

**Subdomain-based routing:**

```yaml
ingress:
  hosts:
    - host: api.forge.example.com
      paths:
        - path: /
          pathType: Prefix
          service: api
          port: 8080
    - host: vfs.forge.example.com
      paths:
        - path: /
          pathType: Prefix
          service: vfs
          port: 8083
    - host: runner.forge.example.com
      paths:
        - path: /
          pathType: Prefix
          service: runner
          port: 8081
```

**With AWS ALB:**

```yaml
ingress:
  className: alb
  annotations:
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/certificate-arn: arn:aws:acm:region:account:certificate/cert-id
    alb.ingress.kubernetes.io/listen-ports: '[{"HTTPS":443}]'
    alb.ingress.kubernetes.io/ssl-redirect: "443"
    alb.ingress.kubernetes.io/healthcheck-path: /healthz
```

### Autoscaling

| Parameter | Description | Default |
|-----------|-------------|---------|
| `hpa.enabled` | Enable HPA | `true` |
| `hpa.api.minReplicas` | API min replicas | `3` |
| `hpa.api.maxReplicas` | API max replicas | `10` |
| `hpa.api.targetCPUUtilization` | API CPU target % | `70` |
| `hpa.runner.minReplicas` | Runner min replicas | `2` |
| `hpa.runner.maxReplicas` | Runner max replicas | `8` |
| `hpa.runner.targetCPUUtilization` | Runner CPU target % | `75` |
| `hpa.vfs.minReplicas` | VFS min replicas | `2` |
| `hpa.vfs.maxReplicas` | VFS max replicas | `6` |

### Pod Disruption Budget

| Parameter | Description | Default |
|-----------|-------------|---------|
| `podDisruptionBudget.enabled` | Enable PDB | `true` |
| `podDisruptionBudget.api.minAvailable` | API min available pods | `2` |
| `podDisruptionBudget.runner.minAvailable` | Runner min available pods | `1` |
| `podDisruptionBudget.brain.maxUnavailable` | Brain max unavailable | `0` |
| `podDisruptionBudget.vfs.minAvailable` | VFS min available pods | `1` |

### Network Policy

| Parameter | Description | Default |
|-----------|-------------|---------|
| `networkPolicy.enabled` | Enable NetworkPolicy | `true` |
| `networkPolicy.defaultDeny` | Default deny all traffic | `true` |

### Monitoring

| Parameter | Description | Default |
|-----------|-------------|---------|
| `serviceMonitor.enabled` | Enable Prometheus ServiceMonitor | `true` |
| `serviceMonitor.interval` | Scrape interval | `30s` |
| `serviceMonitor.scrapeTimeout` | Scrape timeout | `10s` |
| `serviceMonitor.labels.release` | Prometheus release label | `prometheus` |

### Secrets

| Parameter | Description | Default |
|-----------|-------------|---------|
| `secrets.create` | Create secret resource | `true` |
| `secrets.jwtSecret` | JWT signing secret | `""` |
| `secrets.databaseUrl` | PostgreSQL connection URL | `""` |
| `secrets.redisPassword` | Redis password | `""` |

**Use external secrets:**

```yaml
secrets:
  create: false

# Create a K8s secret manually:
kubectl create secret generic civitforge-secrets \
  --from-literal=jwt-secret=$(openssl rand -base64 32) \
  --from-literal=database-url='postgres://user:pass@host:5432/db' \
  --from-literal=redis-password='your-redis-password'
```

### Security Context

The chart enforces security best practices by default:

- `runAsNonRoot: true`
- `runAsUser: 65532` (nonroot)
- `fsGroup: 65532`
- `readOnlyRootFilesystem: true`
- `allowPrivilegeEscalation: false`
- `seccompProfile: RuntimeDefault`
- All capabilities dropped

### Affinity & Tolerations

```yaml
tolerations:
  - key: "civitforge"
    operator: "Equal"
    value: "true"
    effect: "NoSchedule"

affinity:
  api:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
        - labelSelector:
            matchLabels:
              app.kubernetes.io/component: api
          topologyKey: kubernetes.io/hostname
```

## Upgrading

```bash
helm upgrade civitforge civitforge/civitforge \
  --namespace civitforge \
  --reuse-values \
  --set image.tag=2.3.0
```

### Rolling Back

```bash
helm rollback civitforge <REVISION> --namespace civitforge
```

## Uninstalling

```bash
helm uninstall civitforge --namespace civitforge

# Clean up PVCs (data not deleted automatically)
kubectl delete pvc -l app.kubernetes.io/instance=civitforge -n civitforge
```

## Troubleshooting

### Pod CrashLoopBackOff

```bash
kubectl logs -l app.kubernetes.io/component=api -n civitforge --tail=100
```

### Health Check Failures

```bash
kubectl exec -it <pod-name> -n civitforge -- curl -s http://localhost:8080/healthz
```

### NetworkPolicy Issues

```bash
kubectl get networkpolicies -n civitforge
kubectl describe networkpolicy civitforge-default-deny -n civitforge
```

### HPA Not Scaling

```bash
kubectl get hpa -n civitforge
kubectl describe hpa civitforge-api -n civitforge
```
