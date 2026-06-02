//! Organization domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::OrgId;
use crate::id::UserId;
use crate::visibility::Visibility;

/// Organization representation for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgResponse {
    pub id: OrgId,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub visibility: Visibility,
    pub owner_id: UserId,
    pub member_count: u32,
    pub repo_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new organization.
#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<Visibility>,
}

/// Request to update an organization.
#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<Visibility>,
}
