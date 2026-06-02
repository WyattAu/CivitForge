#![forbid(unsafe_code)]

use std::collections::HashMap;

use anyhow::Result;

use super::client::{QdrantClient, QdrantFilter, QdrantSearchRequest, ScoredPoint};

#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub dense_weight: f32,
    pub sparse_weight: f32,
    pub rerank_top_k: usize,
    pub min_score: f32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            dense_weight: 0.7,
            sparse_weight: 0.3,
            rerank_top_k: 50,
            min_score: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub id: String,
    pub score: f32,
    pub dense_score: f32,
    pub sparse_score: f32,
    pub payload: serde_json::Value,
}

pub struct HybridSearcher {
    qdrant: QdrantClient,
    config: HybridSearchConfig,
}

impl HybridSearcher {
    pub fn new(qdrant: QdrantClient, config: HybridSearchConfig) -> Self {
        Self { qdrant, config }
    }

    pub fn with_defaults(qdrant: QdrantClient) -> Self {
        Self::new(qdrant, HybridSearchConfig::default())
    }

    pub fn config(&self) -> &HybridSearchConfig {
        &self.config
    }

    pub async fn search(
        &self,
        query: &str,
        query_embedding: &[f32],
        top_k: usize,
        filter: Option<QdrantFilter>,
    ) -> Result<Vec<HybridSearchResult>> {
        let fetch_k = self.config.rerank_top_k.max(top_k);

        let dense_request = QdrantSearchRequest {
            vector: query_embedding.to_vec(),
            top_k: fetch_k,
            filter: filter.clone(),
            with_payload: true,
            score_threshold: None,
        };

        let dense_results = self.qdrant.search(dense_request).await?;

        let keywords = extract_keywords(query);
        let mut hybrid_results: Vec<HybridSearchResult> = dense_results
            .into_iter()
            .map(|point| {
                let sparse_score = compute_sparse_score(&point, &keywords);
                let combined = (point.score * self.config.dense_weight)
                    + (sparse_score * self.config.sparse_weight);
                HybridSearchResult {
                    id: point.id,
                    score: combined,
                    dense_score: point.score,
                    sparse_score,
                    payload: point.payload,
                }
            })
            .filter(|r| r.score >= self.config.min_score)
            .collect();

        hybrid_results = self.rerank(hybrid_results, query);

        hybrid_results.truncate(top_k);
        Ok(hybrid_results)
    }

    pub fn rerank(
        &self,
        mut results: Vec<HybridSearchResult>,
        query: &str,
    ) -> Vec<HybridSearchResult> {
        let query_terms: HashMap<String, f32> = extract_keywords(query)
            .into_iter()
            .map(|(term, tf)| {
                let idf = (1.0_f32).max((results.len() as f32 / (1.0 + tf)).ln());
                (term, idf)
            })
            .collect();

        for result in &mut results {
            let payload_text = extract_payload_text(&result.payload);
            let relevance = compute_relevance_score(query, &payload_text, &query_terms);
            result.score = (result.score * 0.8) + (relevance * 0.2);
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

pub fn extract_keywords(text: &str) -> Vec<(String, f32)> {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() > 1)
        .collect();

    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
        "as", "into", "through", "during", "before", "after", "above", "below", "between", "out",
        "off", "over", "under", "again", "further", "then", "once", "here", "there", "when",
        "where", "why", "how", "all", "each", "every", "both", "few", "more", "most", "other",
        "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too", "very",
        "just", "because", "but", "and", "or", "if", "while", "that", "this", "these", "those",
        "it", "its", "my", "your", "his", "her", "their", "our", "what", "which", "who", "whom",
    ]
    .iter()
    .copied()
    .collect();

    let mut freq: HashMap<&str, usize> = HashMap::new();
    for word in &words {
        if !stop_words.contains(word) {
            *freq.entry(word).or_insert(0) += 1;
        }
    }

    let max_freq = freq.values().copied().max().unwrap_or(1) as f32;

    let mut keywords: Vec<(String, f32)> = freq
        .into_iter()
        .map(|(word, count)| {
            let tf = (count as f32) / max_freq;
            (word.to_string(), tf)
        })
        .collect();

    keywords.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    keywords
}

fn compute_sparse_score(point: &ScoredPoint, keywords: &[(String, f32)]) -> f32 {
    let payload_text = extract_payload_text(&point.payload);
    let mut score = 0.0f32;
    let mut matched = 0usize;

    for (keyword, tf) in keywords {
        let lower_payload = payload_text.to_lowercase();
        if lower_payload.contains(keyword.as_str()) {
            let idf = 1.0_f32 + (1.0 / (1.0 + *tf)).ln();
            score += tf * idf;
            matched += 1;
        }
    }

    if keywords.is_empty() {
        return 0.0;
    }

    score / (matched as f32 + 0.5).ln()
}

fn extract_payload_text(payload: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(s) = payload.as_str() {
        text.push_str(s);
    } else if let Some(obj) = payload.as_object() {
        for value in obj.values() {
            if let Some(s) = value.as_str() {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(s);
            }
        }
    }
    text
}

fn compute_relevance_score(
    query: &str,
    payload_text: &str,
    query_terms: &HashMap<String, f32>,
) -> f32 {
    let query_lower = query.to_lowercase();
    let payload_lower = payload_text.to_lowercase();

    if payload_lower.is_empty() {
        return 0.0;
    }

    let exact_match = if payload_lower.contains(&query_lower) {
        1.0
    } else {
        0.0
    };

    let mut term_match_score = 0.0f32;
    let mut matched = 0usize;

    for (term, idf) in query_terms {
        if payload_lower.contains(term.as_str()) {
            term_match_score += idf;
            matched += 1;
        }
    }

    if query_terms.is_empty() {
        return exact_match * 0.5;
    }

    let term_coverage = matched as f32 / query_terms.len() as f32;
    let normalized_term = term_match_score / (query_terms.len() as f32).sqrt();

    (exact_match * 0.3) + (normalized_term * 0.5) + (term_coverage * 0.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_search_config_defaults() {
        let config = HybridSearchConfig::default();
        assert!((config.dense_weight - 0.7).abs() < 0.001);
        assert!((config.sparse_weight - 0.3).abs() < 0.001);
        assert_eq!(config.rerank_top_k, 50);
        assert!((config.min_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_extract_keywords_basic() {
        let keywords = extract_keywords("function parse_request handler");
        let terms: Vec<&str> = keywords.iter().map(|(t, _)| t.as_str()).collect();
        assert!(terms.contains(&"function"));
        assert!(terms.contains(&"parse_request"));
        assert!(terms.contains(&"handler"));
    }

    #[test]
    fn test_extract_keywords_removes_stop_words() {
        let keywords = extract_keywords("the function is a handler");
        let terms: Vec<&str> = keywords.iter().map(|(t, _)| t.as_str()).collect();
        assert!(!terms.contains(&"the"));
        assert!(!terms.contains(&"is"));
        assert!(!terms.contains(&"a"));
        assert!(terms.contains(&"function"));
        assert!(terms.contains(&"handler"));
    }

    #[test]
    fn test_extract_keywords_case_insensitive() {
        let keywords = extract_keywords("Function Parse Handler");
        let terms: Vec<&str> = keywords.iter().map(|(t, _)| t.as_str()).collect();
        assert!(terms.contains(&"function"));
        assert!(terms.contains(&"parse"));
        assert!(terms.contains(&"handler"));
    }

    #[test]
    fn test_extract_keywords_empty() {
        let keywords = extract_keywords("");
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_extract_keywords_single_char_filtered() {
        let keywords = extract_keywords("a b c");
        assert!(keywords.is_empty());
    }

    #[test]
    fn test_compute_sparse_score_matching() {
        let point = ScoredPoint {
            id: "1".into(),
            score: 0.9,
            payload: serde_json::json!({"name": "parse_request", "description": "handles parsing"}),
            vector: None,
        };
        let keywords = vec![("parse_request".into(), 1.0), ("handler".into(), 0.5)];
        let score = compute_sparse_score(&point, &keywords);
        assert!(score > 0.0);
    }

    #[test]
    fn test_compute_sparse_score_no_match() {
        let point = ScoredPoint {
            id: "1".into(),
            score: 0.9,
            payload: serde_json::json!({"name": "something_else"}),
            vector: None,
        };
        let keywords = vec![("parse_request".into(), 1.0)];
        let score = compute_sparse_score(&point, &keywords);
        assert!(score < 0.001);
    }

    #[test]
    fn test_extract_payload_text_string() {
        let payload = serde_json::json!("hello world");
        let text = extract_payload_text(&payload);
        assert_eq!(text, "hello world");
    }

    #[test]
    fn test_extract_payload_text_object() {
        let payload = serde_json::json!({"name": "foo", "desc": "bar"});
        let text = extract_payload_text(&payload);
        assert!(text.contains("foo"));
        assert!(text.contains("bar"));
    }

    #[test]
    fn test_extract_payload_text_null() {
        let payload = serde_json::Value::Null;
        let text = extract_payload_text(&payload);
        assert!(text.is_empty());
    }

    #[test]
    fn test_rerank_combines_scores() {
        let config = HybridSearchConfig::default();
        let qdrant = QdrantClient::new(super::super::client::QdrantConfig::default());
        let searcher = HybridSearcher::new(qdrant, config);

        let results = vec![
            HybridSearchResult {
                id: "1".into(),
                score: 0.9,
                dense_score: 0.9,
                sparse_score: 0.8,
                payload: serde_json::json!({"name": "parse_request_handler"}),
            },
            HybridSearchResult {
                id: "2".into(),
                score: 0.7,
                dense_score: 0.7,
                sparse_score: 0.6,
                payload: serde_json::json!({"name": "something_unrelated"}),
            },
        ];

        let reranked = searcher.rerank(results, "parse request handler");
        assert_eq!(reranked.len(), 2);
        assert!(reranked[0].score > reranked[1].score);
        assert_eq!(reranked[0].id, "1");
    }

    #[test]
    fn test_rerank_empty() {
        let config = HybridSearchConfig::default();
        let qdrant = QdrantClient::new(super::super::client::QdrantConfig::default());
        let searcher = HybridSearcher::new(qdrant, config);
        let results = searcher.rerank(vec![], "test");
        assert!(results.is_empty());
    }

    #[test]
    fn test_relevance_score_exact_match() {
        let terms: HashMap<String, f32> = [("handler".to_string(), 1.0)].into_iter().collect();
        let score = compute_relevance_score("handler", "request handler function", &terms);
        assert!(score > 0.0);
    }

    #[test]
    fn test_relevance_score_no_match() {
        let terms: HashMap<String, f32> = [("handler".to_string(), 1.0)].into_iter().collect();
        let score = compute_relevance_score("handler", "completely different", &terms);
        assert!(score < 0.001);
    }

    #[test]
    fn test_relevance_score_empty_query_terms() {
        let terms: HashMap<String, f32> = HashMap::new();
        let score = compute_relevance_score("handler", "handler", &terms);
        assert!(score > 0.0);
    }

    #[test]
    fn test_hybrid_search_result_fields() {
        let result = HybridSearchResult {
            id: "test".into(),
            score: 0.85,
            dense_score: 0.8,
            sparse_score: 0.9,
            payload: serde_json::json!({"key": "value"}),
        };
        assert_eq!(result.id, "test");
        assert!((result.score - 0.85).abs() < 0.001);
        assert_eq!(result.dense_score, 0.8);
        assert_eq!(result.sparse_score, 0.9);
    }
}
