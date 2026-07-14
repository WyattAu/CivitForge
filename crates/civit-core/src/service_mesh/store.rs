use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ServiceMeshStore {
    pool: PgPool,
}

impl ServiceMeshStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_service(&self, req: CreateServiceRequest) -> Result<ServiceMeshService, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let protocol = req.protocol.unwrap_or_else(|| "http".to_string());
        let metadata = req.metadata.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO service_mesh_services (id, name, description, endpoint, protocol, health_check_url, status, metadata, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.endpoint)
        .bind(&protocol)
        .bind(&req.health_check_url)
        .bind(ServiceStatus::Healthy.to_string())
        .bind(&metadata)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ServiceMeshService {
            id,
            name: req.name,
            description,
            endpoint: req.endpoint,
            protocol,
            health_check_url: req.health_check_url,
            status: ServiceStatus::Healthy,
            metadata,
            created_at: now,
        })
    }

    pub async fn get_service(&self, id: Uuid) -> Result<Option<ServiceMeshService>, sqlx::Error> {
        let row = sqlx::query_as::<_, ServiceRow>(
            r#"SELECT id, name, description, endpoint, protocol, health_check_url, status, metadata, created_at
               FROM service_mesh_services WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ServiceMeshService::from))
    }

    pub async fn get_service_by_name(&self, name: &str) -> Result<Option<ServiceMeshService>, sqlx::Error> {
        let row = sqlx::query_as::<_, ServiceRow>(
            r#"SELECT id, name, description, endpoint, protocol, health_check_url, status, metadata, created_at
               FROM service_mesh_services WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ServiceMeshService::from))
    }

    pub async fn list_services(&self, limit: i64, offset: i64) -> Result<Vec<ServiceMeshService>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ServiceRow>(
            r#"SELECT id, name, description, endpoint, protocol, health_check_url, status, metadata, created_at
               FROM service_mesh_services ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ServiceMeshService::from).collect())
    }

    pub async fn update_service(&self, id: Uuid, req: UpdateServiceRequest) -> Result<ServiceMeshService, sqlx::Error> {
        let mut svc = self.get_service(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(name) = req.name {
            sqlx::query(r#"UPDATE service_mesh_services SET name = $1 WHERE id = $2"#)
                .bind(&name).bind(id).execute(&self.pool).await?;
            svc.name = name;
        }
        if let Some(description) = req.description {
            sqlx::query(r#"UPDATE service_mesh_services SET description = $1 WHERE id = $2"#)
                .bind(&description).bind(id).execute(&self.pool).await?;
            svc.description = description;
        }
        if let Some(endpoint) = req.endpoint {
            sqlx::query(r#"UPDATE service_mesh_services SET endpoint = $1 WHERE id = $2"#)
                .bind(&endpoint).bind(id).execute(&self.pool).await?;
            svc.endpoint = endpoint;
        }
        if let Some(protocol) = req.protocol {
            sqlx::query(r#"UPDATE service_mesh_services SET protocol = $1 WHERE id = $2"#)
                .bind(&protocol).bind(id).execute(&self.pool).await?;
            svc.protocol = protocol;
        }
        if let Some(health_check_url) = req.health_check_url {
            sqlx::query(r#"UPDATE service_mesh_services SET health_check_url = $1 WHERE id = $2"#)
                .bind(&health_check_url).bind(id).execute(&self.pool).await?;
            svc.health_check_url = Some(health_check_url);
        }
        if let Some(status) = req.status {
            sqlx::query(r#"UPDATE service_mesh_services SET status = $1 WHERE id = $2"#)
                .bind(status.to_string()).bind(id).execute(&self.pool).await?;
            svc.status = status;
        }
        if let Some(metadata) = req.metadata {
            sqlx::query(r#"UPDATE service_mesh_services SET metadata = $1 WHERE id = $2"#)
                .bind(&metadata).bind(id).execute(&self.pool).await?;
            svc.metadata = metadata;
        }

        Ok(svc)
    }

    pub async fn delete_service(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM service_mesh_services WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_route(&self, req: CreateRouteRequest) -> Result<ServiceMeshRoute, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let weight = req.weight.unwrap_or(100);
        let headers = req.headers.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO service_mesh_routes (id, path, service_id, weight, headers, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(&req.path)
        .bind(req.service_id)
        .bind(weight)
        .bind(&headers)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ServiceMeshRoute {
            id,
            path: req.path,
            service_id: req.service_id,
            weight,
            headers,
            created_at: now,
        })
    }

    pub async fn get_route(&self, id: Uuid) -> Result<Option<ServiceMeshRoute>, sqlx::Error> {
        let row = sqlx::query_as::<_, RouteRow>(
            r#"SELECT id, path, service_id, weight, headers, created_at
               FROM service_mesh_routes WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ServiceMeshRoute::from))
    }

    pub async fn list_routes(&self, limit: i64, offset: i64) -> Result<Vec<ServiceMeshRoute>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RouteRow>(
            r#"SELECT id, path, service_id, weight, headers, created_at
               FROM service_mesh_routes ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ServiceMeshRoute::from).collect())
    }

    pub async fn list_routes_by_service(&self, service_id: Uuid) -> Result<Vec<ServiceMeshRoute>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RouteRow>(
            r#"SELECT id, path, service_id, weight, headers, created_at
               FROM service_mesh_routes WHERE service_id = $1 ORDER BY created_at"#,
        )
        .bind(service_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ServiceMeshRoute::from).collect())
    }

    pub async fn update_route(&self, id: Uuid, req: UpdateRouteRequest) -> Result<ServiceMeshRoute, sqlx::Error> {
        let mut route = self.get_route(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(path) = req.path {
            sqlx::query(r#"UPDATE service_mesh_routes SET path = $1 WHERE id = $2"#)
                .bind(&path).bind(id).execute(&self.pool).await?;
            route.path = path;
        }
        if let Some(service_id) = req.service_id {
            sqlx::query(r#"UPDATE service_mesh_routes SET service_id = $1 WHERE id = $2"#)
                .bind(service_id).bind(id).execute(&self.pool).await?;
            route.service_id = service_id;
        }
        if let Some(weight) = req.weight {
            sqlx::query(r#"UPDATE service_mesh_routes SET weight = $1 WHERE id = $2"#)
                .bind(weight).bind(id).execute(&self.pool).await?;
            route.weight = weight;
        }
        if let Some(headers) = req.headers {
            sqlx::query(r#"UPDATE service_mesh_routes SET headers = $1 WHERE id = $2"#)
                .bind(&headers).bind(id).execute(&self.pool).await?;
            route.headers = headers;
        }

        Ok(route)
    }

    pub async fn delete_route(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM service_mesh_routes WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_policy(&self, req: CreatePolicyRequest) -> Result<ServiceMeshPolicy, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let config = req.config.unwrap_or(serde_json::json!({}));
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO service_mesh_policies (id, name, description, policy_type, config, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.policy_type)
        .bind(&config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ServiceMeshPolicy {
            id,
            name: req.name,
            description,
            policy_type: req.policy_type,
            config,
            enabled,
            created_at: now,
        })
    }

    pub async fn get_policy(&self, id: Uuid) -> Result<Option<ServiceMeshPolicy>, sqlx::Error> {
        let row = sqlx::query_as::<_, PolicyRow>(
            r#"SELECT id, name, description, policy_type, config, enabled, created_at
               FROM service_mesh_policies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ServiceMeshPolicy::from))
    }

    pub async fn list_policies(&self, limit: i64, offset: i64) -> Result<Vec<ServiceMeshPolicy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"SELECT id, name, description, policy_type, config, enabled, created_at
               FROM service_mesh_policies ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ServiceMeshPolicy::from).collect())
    }

    pub async fn update_policy(&self, id: Uuid, req: UpdatePolicyRequest) -> Result<ServiceMeshPolicy, sqlx::Error> {
        let mut policy = self.get_policy(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(name) = req.name {
            sqlx::query(r#"UPDATE service_mesh_policies SET name = $1 WHERE id = $2"#)
                .bind(&name).bind(id).execute(&self.pool).await?;
            policy.name = name;
        }
        if let Some(description) = req.description {
            sqlx::query(r#"UPDATE service_mesh_policies SET description = $1 WHERE id = $2"#)
                .bind(&description).bind(id).execute(&self.pool).await?;
            policy.description = description;
        }
        if let Some(policy_type) = req.policy_type {
            sqlx::query(r#"UPDATE service_mesh_policies SET policy_type = $1 WHERE id = $2"#)
                .bind(&policy_type).bind(id).execute(&self.pool).await?;
            policy.policy_type = policy_type;
        }
        if let Some(config) = req.config {
            sqlx::query(r#"UPDATE service_mesh_policies SET config = $1 WHERE id = $2"#)
                .bind(&config).bind(id).execute(&self.pool).await?;
            policy.config = config;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE service_mesh_policies SET enabled = $1 WHERE id = $2"#)
                .bind(enabled).bind(id).execute(&self.pool).await?;
            policy.enabled = enabled;
        }

        Ok(policy)
    }

    pub async fn delete_policy(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM service_mesh_policies WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn record_metric(&self, service_id: Uuid, req: CreateMetricRequest) -> Result<ServiceMeshMetric, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let labels = req.labels.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO service_mesh_metrics (id, service_id, metric_name, metric_value, labels, recorded_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(service_id)
        .bind(&req.metric_name)
        .bind(req.metric_value)
        .bind(&labels)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ServiceMeshMetric {
            id,
            service_id,
            metric_name: req.metric_name,
            metric_value: req.metric_value,
            labels,
            recorded_at: now,
        })
    }

    pub async fn get_metrics(&self, service_id: Uuid, metric_name: Option<&str>, limit: i64) -> Result<Vec<ServiceMeshMetric>, sqlx::Error> {
        let rows = match metric_name {
            Some(name) => {
                sqlx::query_as::<_, MetricRow>(
                    r#"SELECT id, service_id, metric_name, metric_value, labels, recorded_at
                       FROM service_mesh_metrics WHERE service_id = $1 AND metric_name = $2
                       ORDER BY recorded_at DESC LIMIT $3"#,
                )
                .bind(service_id)
                .bind(name)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, MetricRow>(
                    r#"SELECT id, service_id, metric_name, metric_value, labels, recorded_at
                       FROM service_mesh_metrics WHERE service_id = $1
                       ORDER BY recorded_at DESC LIMIT $2"#,
                )
                .bind(service_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };
        
        Ok(rows.into_iter().map(ServiceMeshMetric::from).collect())
    }

    pub async fn get_traffic_analysis(&self, service_id: Uuid, time_range: &str) -> Result<TrafficAnalysis, sqlx::Error> {
        let metrics = sqlx::query_as::<_, MetricRow>(
            r#"SELECT id, service_id, metric_name, metric_value, labels, recorded_at
               FROM service_mesh_metrics WHERE service_id = $1 AND recorded_at >= NOW() - $2::interval
               ORDER BY recorded_at DESC"#,
        )
        .bind(service_id)
        .bind(time_range)
        .fetch_all(&self.pool)
        .await?;

        let total_requests: u64 = metrics.iter()
            .filter(|m| m.metric_name == "request_count")
            .map(|m| m.metric_value as u64)
            .sum();

        let success_count: u64 = metrics.iter()
            .filter(|m| m.metric_name == "success_count")
            .map(|m| m.metric_value as u64)
            .sum();

        let error_count: u64 = metrics.iter()
            .filter(|m| m.metric_name == "error_count")
            .map(|m| m.metric_value as u64)
            .sum();

        let latency_sum: f64 = metrics.iter()
            .filter(|m| m.metric_name == "latency_ms")
            .map(|m| m.metric_value)
            .sum();

        let latency_count = metrics.iter()
            .filter(|m| m.metric_name == "latency_ms")
            .count();

        let p99_latency = metrics.iter()
            .filter(|m| m.metric_name == "p99_latency_ms")
            .map(|m| m.metric_value)
            .next()
            .unwrap_or(0.0);

        Ok(TrafficAnalysis {
            service_id,
            time_range: time_range.to_string(),
            total_requests,
            success_rate: if total_requests > 0 { (success_count as f64 / total_requests as f64) * 100.0 } else { 0.0 },
            average_latency_ms: if latency_count > 0 { latency_sum / latency_count as f64 } else { 0.0 },
            p99_latency_ms: p99_latency,
            error_rate: if total_requests > 0 { (error_count as f64 / total_requests as f64) * 100.0 } else { 0.0 },
        })
    }

    pub async fn get_performance_metrics(&self, service_id: Uuid) -> Result<PerformanceMetrics, sqlx::Error> {
        let metrics = sqlx::query_as::<_, MetricRow>(
            r#"SELECT id, service_id, metric_name, metric_value, labels, recorded_at
               FROM service_mesh_metrics WHERE service_id = $1 AND recorded_at >= NOW() - INTERVAL '1 hour'
               ORDER BY recorded_at DESC"#,
        )
        .bind(service_id)
        .fetch_all(&self.pool)
        .await?;

        let request_count: u64 = metrics.iter()
            .filter(|m| m.metric_name == "request_count")
            .map(|m| m.metric_value as u64)
            .sum();

        let error_count: u64 = metrics.iter()
            .filter(|m| m.metric_name == "error_count")
            .map(|m| m.metric_value as u64)
            .sum();

        let response_times: Vec<f64> = metrics.iter()
            .filter(|m| m.metric_name == "response_time_ms")
            .map(|m| m.metric_value)
            .collect();

        let avg_response_time = if response_times.is_empty() { 0.0 } else { response_times.iter().sum::<f64>() / response_times.len() as f64 };

        let mut sorted_times = response_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = if sorted_times.is_empty() { 0.0 } else { sorted_times[sorted_times.len() / 2] };
        let p95 = if sorted_times.is_empty() { 0.0 } else { sorted_times[(sorted_times.len() as f64 * 0.95) as usize] };
        let p99 = if sorted_times.is_empty() { 0.0 } else { sorted_times[(sorted_times.len() as f64 * 0.99) as usize] };

        let throughput: f64 = metrics.iter()
            .filter(|m| m.metric_name == "throughput_rps")
            .map(|m| m.metric_value)
            .next()
            .unwrap_or(0.0);

        Ok(PerformanceMetrics {
            service_id,
            request_count,
            error_count,
            average_response_time_ms: avg_response_time,
            p50_response_time_ms: p50,
            p95_response_time_ms: p95,
            p99_response_time_ms: p99,
            throughput_rps: throughput,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ServiceRow {
    id: Uuid,
    name: String,
    description: String,
    endpoint: String,
    protocol: String,
    health_check_url: Option<String>,
    status: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<ServiceRow> for ServiceMeshService {
    fn from(row: ServiceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            endpoint: row.endpoint,
            protocol: row.protocol,
            health_check_url: row.health_check_url,
            status: row.status.parse().unwrap_or(ServiceStatus::Healthy),
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RouteRow {
    id: Uuid,
    path: String,
    service_id: Uuid,
    weight: i32,
    headers: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<RouteRow> for ServiceMeshRoute {
    fn from(row: RouteRow) -> Self {
        Self {
            id: row.id,
            path: row.path,
            service_id: row.service_id,
            weight: row.weight,
            headers: row.headers,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    name: String,
    description: String,
    policy_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<PolicyRow> for ServiceMeshPolicy {
    fn from(row: PolicyRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            policy_type: row.policy_type,
            config: row.config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    id: Uuid,
    service_id: Uuid,
    metric_name: String,
    metric_value: f64,
    labels: serde_json::Value,
    recorded_at: chrono::DateTime<Utc>,
}

impl From<MetricRow> for ServiceMeshMetric {
    fn from(row: MetricRow) -> Self {
        Self {
            id: row.id,
            service_id: row.service_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            labels: row.labels,
            recorded_at: row.recorded_at,
        }
    }
}
