# CivitForge Monitoring Setup

Comprehensive monitoring configuration for CivitForge production deployments.

## Overview

CivitForge exposes metrics via OTLP exporter (built-in) and structured logs via `tracing`. This document covers Prometheus, Grafana, alerting, log aggregation, and distributed tracing setup.

## Prometheus Configuration

### Installation (kube-prometheus-stack)

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set grafana.enabled=true \
  --set alertmanager.enabled=true
```

### ServiceMonitor

The Helm chart includes a ServiceMonitor when `serviceMonitor.enabled: true` (default). It scrapes all four core services:

```yaml
# values.yaml
serviceMonitor:
  enabled: true
  interval: 30s
  scrapeTimeout: 10s
  labels:
    release: prometheus  # Must match Prometheus operator selector
```

### Custom Prometheus Scrape Config

If not using the operator, add to your Prometheus config:

```yaml
scrape_configs:
  - job_name: civitforge-api
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - civitforge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_component]
        regex: api
        action: keep
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        target_label: __address__
        regex: (.+)
        replacement: ${1}

  - job_name: civitforge-runner
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - civitforge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_component]
        regex: runner
        action: keep

  - job_name: civitforge-brain
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - civitforge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_component]
        regex: brain
        action: keep

  - job_name: civitforge-vfs
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - civitforge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_component]
        regex: vfs
        action: keep
```

### Key Metrics to Scrape

CivitForge exposes these metric types via OTLP:

| Metric | Type | Description |
|--------|------|-------------|
| `civit_http_requests_total` | Counter | Total HTTP requests |
| `civit_http_request_duration_seconds` | Histogram | Request latency |
| `civit_http_requests_in_flight` | Gauge | Current in-flight requests |
| `civit_db_pool_connections` | Gauge | Database connection pool |
| `civit_git_operations_total` | Counter | Git operations (clone/push/pull) |
| `civit_pipeline_runs_total` | Counter | Pipeline execution count |
| `civit_pipeline_duration_seconds` | Histogram | Pipeline execution time |
| `civit_runner_active_jobs` | Gauge | Active runner jobs |
| `civit_vfs_operations_total` | Counter | VFS I/O operations |
| `civit_federation_requests_total` | Counter | Federation API calls |
| `process_cpu_seconds_total` | Counter | Process CPU usage |
| `process_resident_memory_bytes` | Gauge | Process memory usage |

## Grafana Dashboard Setup

### Import Dashboards

1. Access Grafana: `kubectl port-forward svc/prometheus-grafana 3000:80 -n monitoring`
2. Login: admin / (password from `kubectl get secret -n monitoring`)
3. Import dashboards via + > Import

### CivitForge API Dashboard

Create dashboard with these panels:

**Request Rate:**

```promql
sum(rate(civit_http_requests_total{namespace="civitforge"}[5m])) by (component)
```

**Error Rate:**

```promql
sum(rate(civit_http_requests_total{namespace="civitforge", status=~"5.."}[5m])) by (component)
/
sum(rate(civit_http_requests_total{namespace="civitforge"}[5m])) by (component)
```

**P50/P95/P99 Latency:**

```promql
histogram_quantile(0.50, sum(rate(civit_http_request_duration_seconds_bucket{namespace="civitforge"}[5m])) by (le, component))
histogram_quantile(0.95, sum(rate(civit_http_request_duration_seconds_bucket{namespace="civitforge"}[5m])) by (le, component))
histogram_quantile(0.99, sum(rate(civit_http_request_duration_seconds_bucket{namespace="civitforge"}[5m])) by (le, component))
```

**In-Flight Requests:**

```promql
sum(civit_http_requests_in_flight{namespace="civitforge"}) by (component)
```

**Database Connection Pool:**

```promql
civit_db_pool_connections{namespace="civitforge", state="active"}
civit_db_pool_connections{namespace="civitforge", state="idle"}
```

### CivitForge Runner Dashboard

**Pipeline Run Rate:**

```promql
sum(rate(civit_pipeline_runs_total{namespace="civitforge"}[5m])) by (status)
```

**Pipeline Duration:**

```promql
histogram_quantile(0.95, sum(rate(civit_pipeline_duration_seconds_bucket{namespace="civitforge"}[5m])) by (le))
```

**Active Runner Jobs:**

```promql
civit_runner_active_jobs{namespace="civitforge"}
```

### CivitForge VFS Dashboard

**VFS Operations Rate:**

```promql
sum(rate(civit_vfs_operations_total{namespace="civitforge"}[5m])) by (operation)
```

**VFS Error Rate:**

```promql
sum(rate(civit_vfs_operations_total{namespace="civitforge", status="error"}[5m])) by (operation)
```

### Kubernetes Infrastructure Dashboard

```promql
# Pod CPU Usage
sum(rate(container_cpu_usage_seconds_total{namespace="civitforge"}[5m])) by (pod)

# Pod Memory Usage
sum(container_memory_working_set_bytes{namespace="civitforge"}) by (pod)

# Pod Network I/O
sum(rate(container_network_receive_bytes_total{namespace="civitforge"}[5m])) by (pod)
sum(rate(container_network_transmit_bytes_total{namespace="civitforge"}[5m])) by (pod)
```

### Resource Usage Summary

```promql
# CPU vs Limits
sum(rate(container_cpu_usage_seconds_total{namespace="civitforge"}[5m])) by (pod)
/
sum(kube_pod_container_resource_limits{namespace="civitforge", resource="cpu"}) by (pod)

# Memory vs Limits
sum(container_memory_working_set_bytes{namespace="civitforge"}) by (pod)
/
sum(kube_pod_container_resource_limits{namespace="civitforge", resource="memory"}) by (pod)
```

## Alert Rules

### PrometheusRule Resource

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: civitforge-alerts
  namespace: monitoring
  labels:
    release: prometheus
spec:
  groups:
    - name: civitforge
      rules:
        # High Error Rate
        - alert: CivitForgeHighErrorRate
          expr: |
            sum(rate(civit_http_requests_total{namespace="civitforge", status=~"5.."}[5m]))
            / sum(rate(civit_http_requests_total{namespace="civitforge"}[5m])) > 0.05
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "CivitForge error rate > 5%"
            description: "Error rate is {{ $value | humanizePercentage }} over the last 5 minutes"

        # High Latency
        - alert: CivitForgeHighLatency
          expr: |
            histogram_quantile(0.95,
              sum(rate(civit_http_request_duration_seconds_bucket{namespace="civitforge"}[5m])) by (le)
            ) > 2
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "CivitForge p95 latency > 2s"
            description: "P95 latency is {{ $value }}s"

        # API Pods Not Ready
        - alert: CivitForgeAPINotReady
          expr: |
            kube_deployment_status_replicas_available{namespace="civitforge", deployment=~".*-api"} < 2
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "CivitForge API pods not ready"
            description: "Only {{ $value }} API pods are ready"

        # Database Connection Pool Exhausted
        - alert: CivitForgeDBPoolExhausted
          expr: |
            civit_db_pool_connections{namespace="civitforge", state="active"} > 18
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "CivitForge DB connection pool near exhaustion"
            description: "{{ $value }} active connections (limit 20)"

        # Runner Jobs Stuck
        - alert: CivitForgeRunnerJobsStuck
          expr: |
            civit_runner_active_jobs{namespace="civitforge"} > 0
            and
            increase(civit_pipeline_runs_total{namespace="civitforge", status="completed"}[30m]) == 0
          for: 30m
          labels:
            severity: warning
          annotations:
            summary: "CivitForge runner jobs may be stuck"
            description: "Active jobs for 30m with no completions"

        # Pod Restarting
        - alert: CivitForgePodRestarting
          expr: |
            increase(kube_pod_container_status_restarts_total{namespace="civitforge"}[1h]) > 3
          labels:
            severity: warning
          annotations:
            summary: "CivitForge pod restarting frequently"
            description: "Pod {{ $labels.pod }} restarted {{ $value }} times in 1h"

        # High Memory Usage
        - alert: CivitForgeHighMemory
          expr: |
            container_memory_working_set_bytes{namespace="civitforge"}
            / kube_pod_container_resource_limits{namespace="civitforge", resource="memory"} > 0.9
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "CivitForge pod memory > 90% of limit"
            description: "Pod {{ $labels.pod }} memory usage is {{ $value | humanizePercentage }}"

        # Federation Errors
        - alert: CivitForgeFederationErrors
          expr: |
            sum(rate(civit_federation_requests_total{namespace="civitforge", status="error"}[5m])) > 0.1
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "CivitForge federation errors"
            description: "Federation error rate is {{ $value }}/s"

        # PostgreSQL Down
        - alert: CivitForgePostgresDown
          expr: pg_up{namespace="civitforge"} == 0
          for: 1m
          labels:
            severity: critical
          annotations:
            summary: "PostgreSQL is down"
            description: "PostgreSQL instance in civitforge namespace is unreachable"

        # Redis Down
        - alert: CivitForgeRedisDown
          expr: redis_up{namespace="civitforge"} == 0
          for: 1m
          labels:
            severity: critical
          annotations:
            summary: "Redis is down"
            description: "Redis instance in civitforge namespace is unreachable"
```

### Alertmanager Config

```yaml
route:
  group_by: ['alertname', 'namespace']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: slack-critical
  routes:
    - match:
        severity: critical
      receiver: pagerduty-critical
    - match:
        severity: warning
      receiver: slack-warning

receivers:
  - name: slack-critical
    slack_configs:
      - api_url: https://hooks.slack.com/services/xxx/yyy/zzz
        channel: '#civitforge-critical'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: slack-warning
    slack_configs:
      - api_url: https://hooks.slack.com/services/xxx/yyy/zzz
        channel: '#civitforge-warnings'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: pagerduty-critical
    pagerduty_configs:
      - service_key: YOUR_PAGERDUTY_KEY
        description: '{{ .GroupLabels.alertname }}'
```

## Log Aggregation Setup

### Structured Logging

CivitForge uses `tracing` with structured JSON output. Configure via environment:

```yaml
configMap:
  data:
    RUST_LOG: "civit_core=info,tower_http=info"
```

### Fluent Bit Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-civitforge
  namespace: logging
data:
  civitforge.conf: |
    [INPUT]
      Name              tail
      Path              /var/log/containers/*civitforge*.log
      Parser            docker
      Tag               civitforge.*
      Mem_Buf_Limit     5MB
      Refresh_Interval  5

    [FILTER]
      Name              parser
      Match             civitforge.*
      Key_Name          log
      Parser            json

    [FILTER]
      Name              modify
      Match             civitforge.*
      Add               cluster production
      Add               environment prod

    [OUTPUT]
      Name              es
      Match             civitforge.*
      Host              elasticsearch.logging.svc.cluster.local
      Port              9200
      Index             civitforge
      Type              _doc
      Suppress_Type_Name On
      Logstash_Format   On
      Logstash_Prefix   civitforge
```

### Loki Configuration

For Grafana Loki:

```yaml
scrape_configs:
  - job_name: civitforge
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - civitforge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_component]
        target_label: component
      - source_labels: [__meta_kubernetes_namespace]
        target_label: namespace

# LogQL queries
# All errors:
{namespace="civitforge"} |= "level=error"

# API errors:
{namespace="civitforge", component="api"} | json | level="error"

# Slow requests:
{namespace="civitforge"} | json | duration > 2s

# Pipeline failures:
{namespace="civitforge", component="runner"} | json | status="failed"
```

## Distributed Tracing Setup

### OTLP Collector

CivitForge has a built-in OTLP exporter. Configure the collector endpoint:

```yaml
configMap:
  data:
    OTEL_EXPORTER_OTLP_ENDPOINT: "http://otel-collector.observability.svc.cluster.local:4317"
    OTEL_SERVICE_NAME: "civitforge"
    OTEL_RESOURCE_ATTRIBUTES: "deployment.environment=production,service.version=2.2.0"
```

### OpenTelemetry Collector Config

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: otel-collector-config
  namespace: observability
data:
  config.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
          http:
            endpoint: 0.0.0.0:4318

    processors:
      batch:
        timeout: 5s
        send_batch_size: 1024
      memory_limiter:
        check_interval: 5s
        limit_mib: 512
        spike_limit_mib: 128

    exporters:
      jaeger:
        endpoint: jaeger-collector.observability.svc.cluster.local:14250
        tls:
          insecure: true
      prometheus:
        endpoint: 0.0.0.0:8889

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [memory_limiter, batch]
          exporters: [jaeger]
        metrics:
          receivers: [otlp]
          processors: [memory_limiter, batch]
          exporters: [prometheus]
```

### Jaeger

```bash
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/crds/jaegertracing.io_jaegers_crd.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/service_account.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/role.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/role_binding.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/operator.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/cluster_role.yaml
kubectl apply -f https://raw.githubusercontent.com/jaegertracing/jaeger-operator/main/deploy/cluster_role_binding.yaml
```

```yaml
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: civitforge
  namespace: observability
spec:
  strategy: production
  collector:
    maxReplicas: 3
    resources:
      limits:
        memory: 512Mi
        cpu: 500m
  query:
    replicas: 2
  storage:
    type: elasticsearch
    options:
      es:
        server-urls: https://elasticsearch.logging.svc.cluster.local:9200
        index-prefix: civitforge
```

## Metrics Export

### To External Monitoring Systems

**Datadog:**

```yaml
# Use OTLP exporter with Datadog OTLP endpoint
OTEL_EXPORTER_OTLP_ENDPOINT: "https://otlp-http.datadoghq.com"
OTEL_EXPORTER_OTLP_HEADERS: "dd-api-key=YOUR_API_KEY"
```

**New Relic:**

```yaml
OTEL_EXPORTER_OTLP_ENDPOINT: "https://otlp.nr-data.net"
OTEL_EXPORTER_OTLP_HEADERS: "api-key=YOUR_LICENSE_KEY"
```

**AWS CloudWatch:**

```yaml
# Use ADOT Collector with CloudWatch exporter
OTEL_EXPORTER_OTLP_ENDPOINT: "http://adot-collector.observability.svc.cluster.local:4317"
```

### Exporting Prometheus Metrics

If using the Prometheus exporter in the OTLP collector:

```promql
# Scrape OTLP-exported metrics from the collector
scrape_configs:
  - job_name: civitforge-otel
    static_configs:
      - targets:
          - otel-collector.observability.svc.cluster.local:8889
```

## Dashboard JSON Import

For pre-built dashboards, see the `deploy/monitoring/dashboards/` directory or import from Grafana.com:

| Dashboard | ID | Description |
|-----------|-----|-------------|
| Kubernetes Cluster | 7249 | Overall cluster health |
| Node Exporter | 1860 | Node metrics |
| PostgreSQL | 9628 | Database metrics |
| Redis | 763 | Cache metrics |

## Quick Setup Commands

```bash
# Install monitoring stack
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring --create-namespace \
  --set grafana.adminPassword=$(openssl rand -base64 12)

# Apply alert rules
kubectl apply -f deploy/monitoring/alerts.yaml

# Port-forward Grafana
kubectl port-forward svc/prometheus-grafana 3000:80 -n monitoring

# Port-forward Prometheus
kubectl port-forward svc/prometheus-kube-prometheus-prometheus 9090:9090 -n monitoring

# Check metrics endpoint
kubectl exec -it <civitforge-api-pod> -n civitforge -- curl -s http://localhost:8080/metrics
```
