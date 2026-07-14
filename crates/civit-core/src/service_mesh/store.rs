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
