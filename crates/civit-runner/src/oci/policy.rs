#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Repository-level policy for container image management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPolicy {
    pub repo_id: String,
    pub immutable_tags: bool,
    pub max_tags: usize,
    pub retention_days: u32,
}

impl Default for RepositoryPolicy {
    fn default() -> Self {
        Self {
            repo_id: String::new(),
            immutable_tags: false,
            max_tags: 100,
            retention_days: 90,
        }
    }
}

impl RepositoryPolicy {
    pub fn new(repo_id: impl Into<String>) -> Self {
        Self {
            repo_id: repo_id.into(),
            ..Default::default()
        }
    }

    pub fn with_immutable_tags(mut self) -> Self {
        self.immutable_tags = true;
        self
    }

    pub fn with_max_tags(mut self, max: usize) -> Self {
        self.max_tags = max;
        self
    }

    pub fn with_retention_days(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }

    /// Check if a tag operation is allowed under the current policy.
    pub fn can_tag(&self, current_tag_count: usize, tag_exists: bool) -> TagDecision {
        if self.immutable_tags && tag_exists {
            return TagDecision::Denied {
                reason: "immutable_tags policy prevents overwriting existing tags".into(),
            };
        }
        if !tag_exists && current_tag_count >= self.max_tags {
            return TagDecision::Denied {
                reason: format!(
                    "tag limit reached ({}/{})",
                    current_tag_count, self.max_tags
                ),
            };
        }
        TagDecision::Allowed
    }

    /// Determine which tags should be garbage collected based on retention.
    pub fn tags_for_gc(
        &self,
        tags: &[TagWithAge],
    ) -> Vec<String> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(self.retention_days as i64);
        tags.iter()
            .filter(|t| t.created_at < cutoff && !t.immutable)
            .map(|t| t.name.clone())
            .collect()
    }
}

/// Result of a tag operation policy check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagDecision {
    Allowed,
    Denied { reason: String },
}

/// Tag metadata for GC evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithAge {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub immutable: bool,
}

/// Manages repository policies across all containers.
pub struct PolicyManager {
    policies: dashmap::DashMap<String, RepositoryPolicy>,
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            policies: dashmap::DashMap::new(),
        }
    }

    /// Set or update a policy for a repository.
    pub fn set_policy(&self, policy: RepositoryPolicy) {
        self.policies
            .insert(policy.repo_id.clone(), policy);
    }

    /// Get the policy for a repository, returning default if none set.
    pub fn get_policy(&self, repo_id: &str) -> RepositoryPolicy {
        self.policies
            .get(repo_id)
            .map(|p| p.value().clone())
            .unwrap_or_else(|| RepositoryPolicy::new(repo_id))
    }

    /// Remove a policy for a repository.
    pub fn remove_policy(&self, repo_id: &str) -> bool {
        self.policies.remove(repo_id).is_some()
    }

    /// List all configured policies.
    pub fn list_policies(&self) -> Vec<RepositoryPolicy> {
        self.policies.iter().map(|r| r.value().clone()).collect()
    }

    /// Check if a tag operation is allowed.
    pub fn check_tag(
        &self,
        repo_id: &str,
        current_tag_count: usize,
        tag_exists: bool,
    ) -> TagDecision {
        let policy = self.get_policy(repo_id);
        policy.can_tag(current_tag_count, tag_exists)
    }

    /// Get tags eligible for garbage collection.
    pub fn gc_candidates(&self, repo_id: &str, tags: &[TagWithAge]) -> Vec<String> {
        let policy = self.get_policy(repo_id);
        policy.tags_for_gc(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let p = RepositoryPolicy::new("repo1");
        assert!(!p.immutable_tags);
        assert_eq!(p.max_tags, 100);
        assert_eq!(p.retention_days, 90);
    }

    #[test]
    fn test_builder() {
        let p = RepositoryPolicy::new("r")
            .with_immutable_tags()
            .with_max_tags(50)
            .with_retention_days(30);
        assert!(p.immutable_tags);
        assert_eq!(p.max_tags, 50);
        assert_eq!(p.retention_days, 30);
    }

    #[test]
    fn test_can_tag_allowed() {
        let p = RepositoryPolicy::new("r");
        assert!(matches!(p.can_tag(10, false), TagDecision::Allowed));
    }

    #[test]
    fn test_can_tag_immutable_denied() {
        let p = RepositoryPolicy::new("r").with_immutable_tags();
        assert!(matches!(p.can_tag(5, true), TagDecision::Denied { .. }));
    }

    #[test]
    fn test_can_tag_immutable_new_ok() {
        let p = RepositoryPolicy::new("r").with_immutable_tags();
        assert!(matches!(p.can_tag(5, false), TagDecision::Allowed));
    }

    #[test]
    fn test_can_tag_limit_denied() {
        let p = RepositoryPolicy::new("r").with_max_tags(3);
        assert!(matches!(p.can_tag(3, false), TagDecision::Denied { .. }));
    }

    #[test]
    fn test_tags_for_gc() {
        let p = RepositoryPolicy::new("r").with_retention_days(7);
        let now = chrono::Utc::now();
        let tags = vec![
            TagWithAge {
                name: "old".into(),
                created_at: now - chrono::Duration::days(10),
                immutable: false,
            },
            TagWithAge {
                name: "new".into(),
                created_at: now - chrono::Duration::days(1),
                immutable: false,
            },
            TagWithAge {
                name: "old-immutable".into(),
                created_at: now - chrono::Duration::days(10),
                immutable: true,
            },
        ];
        let gc = p.tags_for_gc(&tags);
        assert_eq!(gc.len(), 1);
        assert_eq!(gc[0], "old");
    }

    #[test]
    fn test_policy_manager_set_get() {
        let mgr = PolicyManager::new();
        let policy = RepositoryPolicy::new("r1").with_max_tags(10);
        mgr.set_policy(policy);
        let got = mgr.get_policy("r1");
        assert_eq!(got.max_tags, 10);
    }

    #[test]
    fn test_policy_manager_default_for_missing() {
        let mgr = PolicyManager::new();
        let got = mgr.get_policy("nonexistent");
        assert_eq!(got.max_tags, 100);
    }

    #[test]
    fn test_policy_manager_remove() {
        let mgr = PolicyManager::new();
        mgr.set_policy(RepositoryPolicy::new("r1"));
        assert!(mgr.remove_policy("r1"));
        assert!(!mgr.remove_policy("r1"));
    }

    #[test]
    fn test_policy_manager_list() {
        let mgr = PolicyManager::new();
        mgr.set_policy(RepositoryPolicy::new("r1"));
        mgr.set_policy(RepositoryPolicy::new("r2"));
        assert_eq!(mgr.list_policies().len(), 2);
    }

    #[test]
    fn test_policy_manager_check_tag() {
        let mgr = PolicyManager::new();
        let policy = RepositoryPolicy::new("r1").with_immutable_tags();
        mgr.set_policy(policy);
        assert!(matches!(
            mgr.check_tag("r1", 5, true),
            TagDecision::Denied { .. }
        ));
        assert!(matches!(mgr.check_tag("r1", 5, false), TagDecision::Allowed));
    }

    #[test]
    fn test_policy_manager_gc_candidates() {
        let mgr = PolicyManager::new();
        let policy = RepositoryPolicy::new("r1").with_retention_days(7);
        mgr.set_policy(policy);
        let now = chrono::Utc::now();
        let tags = vec![TagWithAge {
            name: "stale".into(),
            created_at: now - chrono::Duration::days(30),
            immutable: false,
        }];
        let gc = mgr.gc_candidates("r1", &tags);
        assert_eq!(gc, vec!["stale".to_string()]);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let p = RepositoryPolicy::new("r").with_immutable_tags();
        let json = serde_json::to_string(&p).unwrap();
        let de: RepositoryPolicy = serde_json::from_str(&json).unwrap();
        assert!(de.immutable_tags);
    }
}
