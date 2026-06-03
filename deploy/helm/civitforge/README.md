# CivitForge Helm Chart

## Prerequisites

- Kubernetes 1.30+
- Helm 3.14+
- Ingress controller (nginx recommended)
- cert-manager (for TLS)

## Installation

```bash
helm repo add civitforge https://charts.civitforge.io
helm repo update

helm install civitforge civitforge/civitforge \
  --namespace civitforge \
  --create-namespace \
  --set secrets.jwtSecret="<jwt-secret>" \
  --set secrets.databaseUrl="postgres://civitforge:password@postgres:5432/civitforge" \
  --set secrets.redisPassword="<redis-password>"
```

## Configuration

### Core Services

| Parameter | Description | Default |
|-----------|-------------|---------|
| `api.replicas` | Number of API replicas | 3 |
| `runner.replicas` | Number of runner replicas | 2 |
| `brain.replicas` | Number of brain replicas | 1 |
| `vfs.replicas` | Number of VFS replicas | 2 |
| `api.resources.limits.cpu` | API CPU limit | 1 |
| `api.resources.limits.memory` | API memory limit | 512Mi |

### Ingress

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | true |
| `ingress.hosts[0].host` | Hostname | forge.example.com |
| `ingress.tls[0].secretName` | TLS secret name | civitforge-tls |

### Autoscaling

| Parameter | Description | Default |
|-----------|-------------|---------|
| `hpa.api.maxReplicas` | Max API replicas | 10 |
| `hpa.runner.maxReplicas` | Max runner replicas | 8 |
| `hpa.api.targetCPUUtilization` | API CPU target | 70 |

### Network Policy

| Parameter | Description | Default |
|-----------|-------------|---------|
| `networkPolicy.enabled` | Enable NetworkPolicy | true |
| `networkPolicy.defaultDeny` | Default deny all traffic | true |

## Upgrading

```bash
helm upgrade civitforge civitforge/civitforge \
  --namespace civitforge \
  --reuse-values
```

## Uninstalling

```bash
helm uninstall civitforge --namespace civitforge
```

## Architecture

The chart deploys four core services:

- **api**: HTTP/WebSocket API server (port 8080)
- **runner**: CI/CD pipeline execution engine (port 8088)
- **brain**: AI/ML services (port 8082)
- **vfs**: Virtual file system with gRPC (port 9090)

Network policies enforce zero-trust communication between services.
