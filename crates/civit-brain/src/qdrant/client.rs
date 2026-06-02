#![forbid(unsafe_code)]

use std::time::Duration;

use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub collection_name: String,
    pub vector_size: usize,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".into(),
            api_key: None,
            timeout: Duration::from_secs(30),
            collection_name: "civitforge".into(),
            vector_size: 1536,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QdrantClient {
    config: QdrantConfig,
    http_client: reqwest::Client,
}

impl QdrantClient {
    pub fn new(config: QdrantConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config,
            http_client,
        }
    }

    pub fn config(&self) -> &QdrantConfig {
        &self.config
    }

    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/healthz", self.config.url);
        let resp = self.http_client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }

    pub async fn create_collection(&self) -> Result<()> {
        let url = format!(
            "{}/collections/{}",
            self.config.url, self.config.collection_name
        );
        let body = serde_json::json!({
            "vectors": {
                "size": self.config.vector_size,
                "distance": "Cosine"
            }
        });
        let mut req = self.http_client.put(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("create_collection failed: {status} - {text}");
        }
        Ok(())
    }

    pub async fn delete_collection(&self) -> Result<()> {
        let url = format!(
            "{}/collections/{}",
            self.config.url, self.config.collection_name
        );
        let mut req = self.http_client.delete(&url);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("delete_collection failed: {status} - {text}");
        }
        Ok(())
    }

    pub async fn collection_info(&self) -> Result<QdrantCollection> {
        let url = format!(
            "{}/collections/{}",
            self.config.url, self.config.collection_name
        );
        let mut req = self.http_client.get(&url);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("collection_info failed: {status} - {text}");
        }
        let body: serde_json::Value = resp.json().await?;
        let result = body.get("result").cloned().unwrap_or_default();
        QdrantCollection::from_json(&result)
    }

    pub async fn upsert_points(&self, points: Vec<QdrantPoint>) -> Result<UpsertResult> {
        if points.is_empty() {
            return Ok(UpsertResult {
                status: "ok".into(),
                operation_id: 0,
            });
        }
        let url = format!(
            "{}/collections/{}/points",
            self.config.url, self.config.collection_name
        );
        let payload: Vec<QdrantPointPayload> = points.into_iter().map(|p| p.into()).collect();
        let body = serde_json::json!({
            "points": payload
        });
        let mut req = self.http_client.put(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("upsert_points failed: {status} - {text}");
        }
        let body: serde_json::Value = resp.json().await?;
        let result = body.get("result").cloned().unwrap_or_default();
        Ok(UpsertResult {
            status: result
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .into(),
            operation_id: result
                .get("operation_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        })
    }

    pub async fn search(&self, query: QdrantSearchRequest) -> Result<Vec<ScoredPoint>> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.config.url, self.config.collection_name
        );
        let limit = query.top_k;
        let mut body = serde_json::json!({
            "vector": query.vector,
            "limit": limit,
            "with_payload": query.with_payload,
        });
        if let Some(filter) = query.filter {
            body["filter"] = serde_json::to_value(filter)?;
        }
        if let Some(threshold) = query.score_threshold {
            body["score_threshold"] = serde_json::json!(threshold);
        }
        let mut req = self.http_client.post(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("search failed: {status} - {text}");
        }
        let body: serde_json::Value = resp.json().await?;
        let results = body
            .get("result")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        results.iter().map(ScoredPoint::from_json).collect()
    }

    pub async fn delete_points(&self, filter: QdrantFilter) -> Result<usize> {
        let url = format!(
            "{}/collections/{}/points/delete",
            self.config.url, self.config.collection_name
        );
        let body = serde_json::json!({
            "filter": serde_json::to_value(filter)?
        });
        let mut req = self.http_client.post(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("delete_points failed: {status} - {text}");
        }
        let body: serde_json::Value = resp.json().await?;
        let result = body.get("result").cloned().unwrap_or_default();
        let deleted = result.get("deleted").and_then(|v| v.as_u64()).unwrap_or(0);
        let ok = result.get("status").and_then(|v| v.as_str()) == Some("ok");
        Ok(if ok || deleted > 0 {
            deleted as usize
        } else {
            0
        })
    }

    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let url = format!("{}/collections", self.config.url);
        let mut req = self.http_client.get(&url);
        if let Some(key) = &self.config.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("list_collections failed: {status} - {text}");
        }
        let body: serde_json::Value = resp.json().await?;
        let collections = body
            .get("result")
            .and_then(|v| v.get("collections"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(collections
            .iter()
            .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(String::from))
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct QdrantPointPayload {
    id: String,
    vector: Vec<f32>,
    payload: serde_json::Value,
}

impl From<QdrantPoint> for QdrantPointPayload {
    fn from(p: QdrantPoint) -> Self {
        Self {
            id: p.id,
            vector: p.vector,
            payload: p.payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpsertResult {
    pub status: String,
    pub operation_id: u64,
}

#[derive(Debug, Clone)]
pub struct QdrantSearchRequest {
    pub vector: Vec<f32>,
    pub top_k: usize,
    pub filter: Option<QdrantFilter>,
    pub with_payload: bool,
    pub score_threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QdrantFilter {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub must: Vec<FilterCondition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub should: Vec<FilterCondition>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub must_not: Vec<FilterCondition>,
}

impl QdrantFilter {
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            should: Vec::new(),
            must_not: Vec::new(),
        }
    }
}

impl Default for QdrantFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FilterCondition {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_any: Option<Vec<serde_json::Value>>,
}

impl FilterCondition {
    pub fn match_eq(key: &str, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            match_value: Some(value),
            match_any: None,
        }
    }

    pub fn match_any(key: &str, values: Vec<serde_json::Value>) -> Self {
        Self {
            key: key.into(),
            match_value: None,
            match_any: Some(values),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoredPoint {
    pub id: String,
    pub score: f32,
    pub payload: serde_json::Value,
    pub vector: Option<Vec<f32>>,
}

impl ScoredPoint {
    fn from_json(value: &serde_json::Value) -> Result<Self> {
        let id = value
            .get("id")
            .and_then(|v| {
                if v.is_string() {
                    v.as_str().map(String::from)
                } else if v.is_number() {
                    v.as_u64().map(|n| n.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let score = value.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let payload = value
            .get("payload")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let vector = value.get("vector").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|e| e.as_f64().map(|f| f as f32))
                    .collect()
            })
        });
        Ok(Self {
            id,
            score,
            payload,
            vector,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistanceMetric {
    Cosine,
    Euclid,
    Dot,
}

impl DistanceMetric {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "euclid" => Self::Euclid,
            "dot" => Self::Dot,
            _ => Self::Cosine,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionStatus {
    Green,
    Yellow,
    Red,
}

impl CollectionStatus {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "yellow" => Self::Yellow,
            "red" => Self::Red,
            _ => Self::Green,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QdrantCollection {
    pub name: String,
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub vectors_count: u64,
    pub index_threshold: u64,
    pub status: CollectionStatus,
}

impl QdrantCollection {
    fn from_json(value: &serde_json::Value) -> Result<Self> {
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .into();
        let config = value.get("config").cloned().unwrap_or_default();
        let params = config.get("params").cloned().unwrap_or_default();
        let vectors = config.get("vectors").cloned().unwrap_or_default();
        let vector_size = vectors.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let distance = vectors
            .get("distance")
            .and_then(|v| v.as_str())
            .map(DistanceMetric::from_str)
            .unwrap_or(DistanceMetric::Cosine);
        let vectors_count = value
            .get("vectors_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let index_threshold = params
            .get("index_threshold")
            .and_then(|v| v.as_u64())
            .unwrap_or(20000);
        let status = value
            .get("status")
            .and_then(|v| v.as_str())
            .map(CollectionStatus::from_str)
            .unwrap_or(CollectionStatus::Green);
        Ok(Self {
            name,
            vector_size,
            distance,
            vectors_count,
            index_threshold,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> QdrantConfig {
        QdrantConfig {
            url: "http://localhost:6333".into(),
            api_key: None,
            timeout: Duration::from_secs(5),
            collection_name: "test_collection".into(),
            vector_size: 128,
        }
    }

    #[test]
    fn test_default_config() {
        let config = QdrantConfig::default();
        assert_eq!(config.url, "http://localhost:6333");
        assert_eq!(config.vector_size, 1536);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_client_new() {
        let client = QdrantClient::new(test_config());
        assert_eq!(client.config().collection_name, "test_collection");
    }

    #[test]
    fn test_qdrant_point_creation() {
        let point = QdrantPoint {
            id: "test-id".into(),
            vector: vec![0.1, 0.2, 0.3],
            payload: serde_json::json!({"name": "test"}),
        };
        assert_eq!(point.id, "test-id");
        assert_eq!(point.vector.len(), 3);
    }

    #[test]
    fn test_qdrant_filter_new() {
        let filter = QdrantFilter::new();
        assert!(filter.must.is_empty());
        assert!(filter.should.is_empty());
        assert!(filter.must_not.is_empty());
    }

    #[test]
    fn test_qdrant_filter_default() {
        let filter = QdrantFilter::default();
        assert!(filter.must.is_empty());
    }

    #[test]
    fn test_filter_condition_match_eq() {
        let cond = FilterCondition::match_eq("type", serde_json::json!("Function"));
        assert_eq!(cond.key, "type");
        assert!(cond.match_value.is_some());
        assert!(cond.match_any.is_none());
    }

    #[test]
    fn test_filter_condition_match_any() {
        let cond = FilterCondition::match_any(
            "language",
            vec![serde_json::json!("rust"), serde_json::json!("go")],
        );
        assert_eq!(cond.key, "language");
        assert!(cond.match_value.is_none());
        assert_eq!(cond.match_any.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_filter_serialization() {
        let mut filter = QdrantFilter::new();
        filter.must.push(FilterCondition::match_eq(
            "type",
            serde_json::json!("Function"),
        ));
        let json = serde_json::to_value(&filter).unwrap();
        assert!(json.get("must").is_some());
        assert!(json.get("should").is_none());
    }

    #[test]
    fn test_filter_serialization_empty_must_not() {
        let filter = QdrantFilter::new();
        let json = serde_json::to_value(&filter).unwrap();
        assert!(json.get("must_not").is_none());
    }

    #[test]
    fn test_scored_point_from_json() {
        let json = serde_json::json!({
            "id": "123",
            "score": 0.95,
            "payload": {"name": "test"},
            "vector": [0.1, 0.2]
        });
        let point = ScoredPoint::from_json(&json).unwrap();
        assert_eq!(point.id, "123");
        assert!((point.score - 0.95).abs() < 0.001);
        assert_eq!(point.payload["name"], "test");
        assert!(point.vector.is_some());
    }

    #[test]
    fn test_scored_point_from_json_u64_id() {
        let json = serde_json::json!({
            "id": 456,
            "score": 0.8,
            "payload": null
        });
        let point = ScoredPoint::from_json(&json).unwrap();
        assert_eq!(point.id, "456");
    }

    #[test]
    fn test_scored_point_from_json_minimal() {
        let json = serde_json::json!({});
        let point = ScoredPoint::from_json(&json).unwrap();
        assert!(point.id.is_empty());
        assert_eq!(point.score, 0.0);
        assert!(point.vector.is_none());
    }

    #[test]
    fn test_collection_from_json() {
        let json = serde_json::json!({
            "name": "test",
            "config": {
                "params": {"index_threshold": 10000},
                "vectors": {"size": 256, "distance": "Cosine"}
            },
            "vectors_count": 42,
            "status": "green"
        });
        let col = QdrantCollection::from_json(&json).unwrap();
        assert_eq!(col.name, "test");
        assert_eq!(col.vector_size, 256);
        assert_eq!(col.distance, DistanceMetric::Cosine);
        assert_eq!(col.vectors_count, 42);
        assert_eq!(col.index_threshold, 10000);
        assert_eq!(col.status, CollectionStatus::Green);
    }

    #[test]
    fn test_collection_from_json_missing_fields() {
        let json = serde_json::json!({"name": "minimal"});
        let col = QdrantCollection::from_json(&json).unwrap();
        assert_eq!(col.name, "minimal");
        assert_eq!(col.vector_size, 0);
        assert_eq!(col.status, CollectionStatus::Green);
    }

    #[test]
    fn test_distance_metric_parsing() {
        assert_eq!(DistanceMetric::from_str("cosine"), DistanceMetric::Cosine);
        assert_eq!(DistanceMetric::from_str("Cosine"), DistanceMetric::Cosine);
        assert_eq!(DistanceMetric::from_str("euclid"), DistanceMetric::Euclid);
        assert_eq!(DistanceMetric::from_str("dot"), DistanceMetric::Dot);
    }

    #[test]
    fn test_collection_status_parsing() {
        assert_eq!(CollectionStatus::from_str("green"), CollectionStatus::Green);
        assert_eq!(
            CollectionStatus::from_str("yellow"),
            CollectionStatus::Yellow
        );
        assert_eq!(CollectionStatus::from_str("red"), CollectionStatus::Red);
        assert_eq!(
            CollectionStatus::from_str("unknown"),
            CollectionStatus::Green
        );
    }

    #[test]
    fn test_upsert_result() {
        let result = UpsertResult {
            status: "ok".into(),
            operation_id: 42,
        };
        assert_eq!(result.status, "ok");
        assert_eq!(result.operation_id, 42);
    }

    #[test]
    fn test_search_request_building() {
        let req = QdrantSearchRequest {
            vector: vec![0.1, 0.2],
            top_k: 10,
            filter: None,
            with_payload: true,
            score_threshold: None,
        };
        assert_eq!(req.top_k, 10);
        assert!(req.filter.is_none());
    }

    #[test]
    fn test_search_request_with_filter() {
        let filter = QdrantFilter::new();
        let req = QdrantSearchRequest {
            vector: vec![0.1],
            top_k: 5,
            filter: Some(filter),
            with_payload: false,
            score_threshold: Some(0.7),
        };
        assert!(req.filter.is_some());
        assert_eq!(req.score_threshold, Some(0.7));
    }
}
