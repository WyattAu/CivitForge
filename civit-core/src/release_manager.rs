#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub commit_hash: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub checksum_sha256: String,
    pub download_url: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}

pub struct ReleaseManager {
    releases: std::sync::Mutex<Vec<Release>>,
}

impl ReleaseManager {
    pub fn new() -> Self {
        Self {
            releases: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn create_release(&self, release: Release) -> Result<(), String> {
        let mut releases = self.releases.lock().unwrap();
        if releases.iter().any(|r| r.tag_name == release.tag_name) {
            return Err(format!(
                "release with tag '{}' already exists",
                release.tag_name
            ));
        }
        releases.push(release);
        Ok(())
    }

    pub fn get_by_tag(&self, tag: &str) -> Option<Release> {
        self.releases
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.tag_name == tag)
            .cloned()
    }

    pub fn get_latest(&self) -> Option<Release> {
        let releases = self.releases.lock().unwrap();
        releases
            .iter()
            .filter(|r| !r.draft && !r.prerelease)
            .max_by_key(|r| r.created_at)
            .cloned()
    }

    pub fn list_releases(&self, include_drafts: bool) -> Vec<Release> {
        let releases = self.releases.lock().unwrap();
        releases
            .iter()
            .filter(|r| include_drafts || !r.draft)
            .cloned()
            .collect()
    }

    pub fn delete_release(&self, tag: &str) -> bool {
        let mut releases = self.releases.lock().unwrap();
        let before = releases.len();
        releases.retain(|r| r.tag_name != tag);
        releases.len() < before
    }

    pub fn publish_release(&self, tag: &str) -> Result<(), String> {
        let mut releases = self.releases.lock().unwrap();
        let release = releases
            .iter_mut()
            .find(|r| r.tag_name == tag)
            .ok_or_else(|| format!("release '{tag}' not found"))?;
        if !release.draft {
            return Err(format!("release '{tag}' is not a draft"));
        }
        release.draft = false;
        release.published_at = Some(Utc::now());
        Ok(())
    }

    pub fn add_asset(&self, tag: &str, asset: ReleaseAsset) -> Result<(), String> {
        let mut releases = self.releases.lock().unwrap();
        let release = releases
            .iter_mut()
            .find(|r| r.tag_name == tag)
            .ok_or_else(|| format!("release '{tag}' not found"))?;
        release.assets.push(asset);
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.releases.lock().unwrap().len()
    }
}

impl Default for ReleaseManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_release(tag: &str, draft: bool, prerelease: bool) -> Release {
        Release {
            id: format!("rel-{tag}"),
            tag_name: tag.to_string(),
            name: Some(format!("Release {tag}")),
            body: Some(format!("Changes for {tag}")),
            draft,
            prerelease,
            commit_hash: "abc123".to_string(),
            author: "testuser".to_string(),
            created_at: Utc::now(),
            published_at: None,
            assets: Vec::new(),
        }
    }

    fn make_asset(name: &str) -> ReleaseAsset {
        ReleaseAsset {
            id: format!("asset-{name}"),
            name: name.to_string(),
            size_bytes: 1024,
            content_type: "application/octet-stream".to_string(),
            checksum_sha256: "deadbeef".to_string(),
            download_url: Some(format!("https://example.com/{name}")),
            uploaded_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_release() {
        let rm = ReleaseManager::new();
        let release = make_release("v1.0.0", false, false);
        assert!(rm.create_release(release).is_ok());
        assert_eq!(rm.count(), 1);
    }

    #[test]
    fn test_duplicate_tag_rejected() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        let result = rm.create_release(make_release("v1.0.0", false, false));
        assert!(result.is_err());
        assert_eq!(rm.count(), 1);
    }

    #[test]
    fn test_get_by_tag() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v2.0.0", false, false))
            .unwrap();
        let found = rm.get_by_tag("v2.0.0").unwrap();
        assert_eq!(found.tag_name, "v2.0.0");
    }

    #[test]
    fn test_get_by_tag_missing() {
        let rm = ReleaseManager::new();
        assert!(rm.get_by_tag("nonexistent").is_none());
    }

    #[test]
    fn test_get_latest_non_draft() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        rm.create_release(make_release("v2.0.0", false, false))
            .unwrap();
        let latest = rm.get_latest().unwrap();
        assert_eq!(latest.tag_name, "v2.0.0");
    }

    #[test]
    fn test_get_latest_excludes_drafts() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", true, false))
            .unwrap();
        assert!(rm.get_latest().is_none());
    }

    #[test]
    fn test_get_latest_excludes_prereleases() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0-alpha", false, true))
            .unwrap();
        assert!(rm.get_latest().is_none());
    }

    #[test]
    fn test_list_releases_excludes_drafts_by_default() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        rm.create_release(make_release("v2.0.0", true, false))
            .unwrap();
        let list = rm.list_releases(false);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].tag_name, "v1.0.0");
    }

    #[test]
    fn test_list_releases_includes_drafts() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        rm.create_release(make_release("v2.0.0", true, false))
            .unwrap();
        let list = rm.list_releases(true);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_release() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        assert!(rm.delete_release("v1.0.0"));
        assert_eq!(rm.count(), 0);
    }

    #[test]
    fn test_delete_nonexistent_release() {
        let rm = ReleaseManager::new();
        assert!(!rm.delete_release("v999.0.0"));
    }

    #[test]
    fn test_publish_draft() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", true, false))
            .unwrap();
        assert!(rm.publish_release("v1.0.0").is_ok());
        let release = rm.get_by_tag("v1.0.0").unwrap();
        assert!(!release.draft);
        assert!(release.published_at.is_some());
    }

    #[test]
    fn test_publish_non_draft_fails() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        assert!(rm.publish_release("v1.0.0").is_err());
    }

    #[test]
    fn test_add_asset() {
        let rm = ReleaseManager::new();
        rm.create_release(make_release("v1.0.0", false, false))
            .unwrap();
        let asset = make_asset("binary.tar.gz");
        assert!(rm.add_asset("v1.0.0", asset).is_ok());
        let release = rm.get_by_tag("v1.0.0").unwrap();
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "binary.tar.gz");
    }

    #[test]
    fn test_add_asset_nonexistent_release() {
        let rm = ReleaseManager::new();
        let asset = make_asset("binary.tar.gz");
        assert!(rm.add_asset("nonexistent", asset).is_err());
    }

    #[test]
    fn test_release_serialization_roundtrip() {
        let release = make_release("v3.0.0", true, false);
        let json = serde_json::to_string(&release).unwrap();
        let de: Release = serde_json::from_str(&json).unwrap();
        assert_eq!(de.tag_name, "v3.0.0");
        assert_eq!(de.author, "testuser");
        assert!(de.draft);
    }

    #[test]
    fn test_asset_serialization_roundtrip() {
        let asset = make_asset("checksum.sha256");
        let json = serde_json::to_string(&asset).unwrap();
        let de: ReleaseAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "checksum.sha256");
        assert_eq!(de.checksum_sha256, "deadbeef");
    }

    #[test]
    fn test_default_is_empty() {
        let rm = ReleaseManager::default();
        assert_eq!(rm.count(), 0);
        assert!(rm.list_releases(true).is_empty());
    }
}
