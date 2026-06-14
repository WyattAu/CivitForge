use chrono::{TimeZone, Utc};
use civit_shared::{
    error::{ApiError, ApiErrorCode},
    id::{IssueId, OrgId, RepoId, UserId},
    org::{CreateOrgRequest, OrgResponse, UpdateOrgRequest},
    pagination::{Pagination, PaginationParams},
    permissions::{Action, PermissionCheck, Resource},
    repo::{CreateRepoRequest, RepoResponse},
    user::{UserResponse, UserRole},
    visibility::Visibility,
};
use civit_ui::api::types::{
    AuthResponse, CommentResponse, CreateIssueBody, CreateWikiPageBody, IssueResponse,
    ListResponse, SearchResponse, SearchResultItem, SshKeyResponse, UpdateIssueBody,
    UpdateWikiPageBody, WikiPageListItem, WikiPageResponse, WikiRevision,
};

fn test_json<T: serde::de::DeserializeOwned>(json_str: &str) -> T {
    serde_json::from_str(json_str).unwrap()
}

#[test]
fn list_response_roundtrip() {
    let pag = Pagination {
        page: 1,
        per_page: 10,
        total: 25,
        total_pages: 3,
    };
    let resp = ListResponse {
        data: vec![1, 2, 3],
        pagination: pag,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ListResponse<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.data, vec![1, 2, 3]);
    assert_eq!(back.pagination.page, 1);
    assert_eq!(back.pagination.total, 25);
}

#[test]
fn auth_response_deserialize() {
    let json = r#"{"token":"abc123","user":{"id":"550e8400-e29b-41d4-a716-446655440001","username":"alice","email":"a@test.com","display_name":"Alice"}}"#;
    let auth: AuthResponse = test_json(json);
    assert_eq!(auth.token, "abc123");
    assert_eq!(auth.user.username, "alice");
    assert_eq!(auth.user.display_name.as_deref(), Some("Alice"));
}

#[test]
fn auth_response_deserialize_no_display_name() {
    let json = r#"{"token":"tok","user":{"id":"550e8400-e29b-41d4-a716-446655440002","username":"bob","email":"b@test.com","display_name":null}}"#;
    let auth: AuthResponse = test_json(json);
    assert!(auth.user.display_name.is_none());
}

#[test]
fn issue_response_roundtrip() {
    let issue = IssueResponse {
        id: "42".into(),
        number: Some(5),
        title: "Bug fix".into(),
        body: Some("Fix the thing".into()),
        state: "open".into(),
        author: "alice".into(),
        labels: vec!["bug".into(), "urgent".into()],
        created_at: "2024-01-15T10:00:00Z".into(),
        updated_at: "2024-01-16T12:00:00Z".into(),
        comments: vec![],
    };
    let json = serde_json::to_string(&issue).unwrap();
    let back: IssueResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "42");
    assert_eq!(back.number, Some(5));
    assert_eq!(back.labels.len(), 2);
    assert!(back.comments.is_empty());
}

#[test]
fn issue_response_comments_default() {
    let json = r#"{"id":"abc","number":null,"title":"T","body":null,"state":"open","author":"a","labels":[],"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#;
    let issue: IssueResponse = test_json(json);
    assert!(issue.comments.is_empty());
}

#[test]
fn comment_response_roundtrip() {
    let c = CommentResponse {
        id: "10".into(),
        author: "bob".into(),
        body: "Looks good".into(),
        created_at: "2024-02-01T00:00:00Z".into(),
        updated_at: "2024-02-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: CommentResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.author, "bob");
}

#[test]
fn create_issue_body_serialization() {
    let body = CreateIssueBody {
        title: "New bug".into(),
        description: Some("Details here".into()),
    };
    let json = serde_json::to_string(&body).unwrap();
    assert!(json.contains("New bug"));
    assert!(json.contains("Details here"));
}

#[test]
fn create_issue_body_skip_none_description() {
    let body = CreateIssueBody {
        title: "Title".into(),
        description: None,
    };
    let json = serde_json::to_string(&body).unwrap();
    assert!(!json.contains("description"));
}

#[test]
fn update_issue_body_skip_none_fields() {
    let body = UpdateIssueBody {
        title: None,
        body: None,
        state: Some("closed".into()),
    };
    let json = serde_json::to_string(&body).unwrap();
    assert!(!json.contains("title"));
    assert!(!json.contains("body"));
    assert!(json.contains("closed"));
}

#[test]
fn search_response_roundtrip() {
    let resp = SearchResponse {
        results: vec![SearchResultItem {
            id: "1".into(),
            full_name: "org/repo".into(),
            description: Some("A repo".into()),
            stars: 10,
            language: Some("Rust".into()),
        }],
        total: 1,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: SearchResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total, 1);
    assert_eq!(back.results[0].full_name, "org/repo");
}

#[test]
fn wiki_types_roundtrip() {
    let page = WikiPageResponse {
        slug: "getting-started".into(),
        title: "Getting Started".into(),
        content: "Hello world".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&page).unwrap();
    let back: WikiPageResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.slug, "getting-started");

    let list_item = WikiPageListItem {
        slug: "s".into(),
        title: "T".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&list_item).unwrap();
    let back: WikiPageListItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "T");

    let rev = WikiRevision {
        revision: 2,
        slug: "s".into(),
        title: "T".into(),
        content: "C".into(),
        author: "alice".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&rev).unwrap();
    let back: WikiRevision = serde_json::from_str(&json).unwrap();
    assert_eq!(back.revision, 2);
}

#[test]
fn wiki_upsert_serialization() {
    let create = CreateWikiPageBody {
        slug: "new-page".into(),
        title: "New".into(),
        content: "Content".into(),
    };
    let json = serde_json::to_string(&create).unwrap();
    assert!(json.contains("new-page"));

    let update = UpdateWikiPageBody {
        title: Some("Updated".into()),
        content: "New content".into(),
    };
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("Updated"));
    assert!(json.contains("New content"));
}

#[test]
fn ssh_key_response_roundtrip() {
    let key = SshKeyResponse {
        id: "1".into(),
        user_id: "u1".into(),
        key_type: "ed25519".into(),
        public_key: "ssh-ed25519 AAAA...".into(),
        fingerprint: "SHA256:abc".into(),
        label: "my-key".into(),
        created_at: "2024-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&key).unwrap();
    let back: SshKeyResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.label, "my-key");
}

#[test]
fn visibility_serde_lowercase() {
    assert_eq!(
        serde_json::to_string(&Visibility::Public).unwrap(),
        "\"public\""
    );
    assert_eq!(
        serde_json::to_string(&Visibility::Private).unwrap(),
        "\"private\""
    );
    assert_eq!(
        serde_json::to_string(&Visibility::Internal).unwrap(),
        "\"internal\""
    );
}

#[test]
fn visibility_display() {
    assert_eq!(format!("{}", Visibility::Public), "public");
    assert_eq!(format!("{}", Visibility::Internal), "internal");
    assert_eq!(format!("{}", Visibility::Private), "private");
}

#[test]
fn visibility_is_public() {
    assert!(Visibility::Public.is_public());
    assert!(!Visibility::Private.is_public());
    assert!(!Visibility::Internal.is_public());
}

#[test]
fn user_role_serde_snake_case() {
    assert_eq!(
        serde_json::to_string(&UserRole::Owner).unwrap(),
        "\"owner\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Admin).unwrap(),
        "\"admin\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Maintainer).unwrap(),
        "\"maintainer\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Developer).unwrap(),
        "\"developer\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Reporter).unwrap(),
        "\"reporter\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Guest).unwrap(),
        "\"guest\""
    );
}

#[test]
fn user_role_ranking() {
    assert!(UserRole::Owner.rank() > UserRole::Admin.rank());
    assert!(UserRole::Admin.rank() > UserRole::Maintainer.rank());
    assert!(UserRole::Maintainer.rank() > UserRole::Developer.rank());
    assert!(UserRole::Developer.rank() > UserRole::Reporter.rank());
    assert!(UserRole::Reporter.rank() > UserRole::Guest.rank());
}

#[test]
fn user_role_write_admin_permissions() {
    for role in [
        UserRole::Owner,
        UserRole::Admin,
        UserRole::Maintainer,
        UserRole::Developer,
    ] {
        assert!(role.can_write(), "{role:?} should have write access");
    }
    for role in [UserRole::Reporter, UserRole::Guest] {
        assert!(!role.can_write(), "{role:?} should not have write access");
    }
    for role in [UserRole::Owner, UserRole::Admin] {
        assert!(role.can_admin(), "{role:?} should have admin access");
    }
    for role in [
        UserRole::Maintainer,
        UserRole::Developer,
        UserRole::Reporter,
        UserRole::Guest,
    ] {
        assert!(!role.can_admin(), "{role:?} should not have admin access");
    }
}

#[test]
fn user_role_from_str() {
    assert_eq!("owner".parse::<UserRole>(), Ok(UserRole::Owner));
    assert_eq!("guest".parse::<UserRole>(), Ok(UserRole::Guest));
    assert!("unknown".parse::<UserRole>().is_err());
}

#[test]
fn typed_id_serde_transparent() {
    let uid = UserId::nil();
    let json = serde_json::to_string(&uid).unwrap();
    let back: UserId = serde_json::from_str(&json).unwrap();
    assert_eq!(back, uid);
    assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
}

#[test]
fn typed_id_type_safety() {
    let uid = UserId::nil();
    let rid = RepoId::nil();
    let _iid = IssueId::nil();
    assert_eq!(uid.get(), rid.get());
    assert!(std::any::TypeId::of::<UserId>() != std::any::TypeId::of::<RepoId>());
    assert!(std::any::TypeId::of::<RepoId>() != std::any::TypeId::of::<IssueId>());
}

#[test]
fn typed_id_nil() {
    let nil_uuid = UserId::nil().get();
    assert_eq!(nil_uuid.to_string(), "00000000-0000-0000-0000-000000000000");
    assert_eq!(RepoId::nil().get(), nil_uuid);
}

#[test]
fn pagination_params_default() {
    let p = PaginationParams::default();
    assert_eq!(p.effective_per_page(), 20);
    assert_eq!(p.effective_offset(), 0);
}

#[test]
fn pagination_params_page_offset() {
    let p = PaginationParams {
        per_page: Some(10),
        page: Some(3),
        offset: None,
    };
    assert_eq!(p.effective_offset(), 20);
}

#[test]
fn pagination_params_clamp() {
    let p = PaginationParams {
        per_page: Some(200),
        page: None,
        offset: None,
    };
    assert_eq!(p.effective_per_page(), 100);
    let p = PaginationParams {
        per_page: Some(0),
        page: None,
        offset: None,
    };
    assert_eq!(p.effective_per_page(), 1);
}

#[test]
fn pagination_from_total() {
    let params = PaginationParams {
        per_page: Some(10),
        page: Some(2),
        offset: None,
    };
    let pag = Pagination::from_total(25, &params);
    assert_eq!(pag.page, 2);
    assert_eq!(pag.per_page, 10);
    assert_eq!(pag.total, 25);
    assert_eq!(pag.total_pages, 3);
    assert!(pag.has_next());
    assert!(pag.has_prev());
}

#[test]
fn pagination_edge_cases() {
    let params = PaginationParams::default();
    let pag = Pagination::from_total(0, &params);
    assert_eq!(pag.total_pages, 1);
    assert!(!pag.has_next());
    assert!(!pag.has_prev());
}

#[test]
fn api_error_code_http_status() {
    assert_eq!(ApiErrorCode::Unauthorized.http_status(), 401);
    assert_eq!(ApiErrorCode::Forbidden.http_status(), 403);
    assert_eq!(ApiErrorCode::NotFound.http_status(), 404);
    assert_eq!(ApiErrorCode::Conflict.http_status(), 409);
    assert_eq!(ApiErrorCode::ValidationError.http_status(), 422);
    assert_eq!(ApiErrorCode::RateLimited.http_status(), 429);
    assert_eq!(ApiErrorCode::InternalError.http_status(), 500);
    assert_eq!(ApiErrorCode::ServiceUnavailable.http_status(), 503);
}

#[test]
fn api_error_serde_uppercase() {
    assert_eq!(
        serde_json::to_string(&ApiErrorCode::NotFound).unwrap(),
        "\"NOT_FOUND\""
    );
    assert_eq!(
        serde_json::to_string(&ApiErrorCode::ValidationError).unwrap(),
        "\"VALIDATION_ERROR\""
    );
}

#[test]
fn api_error_details_optional() {
    let with_details = ApiError {
        code: ApiErrorCode::ValidationError,
        message: "Bad input".into(),
        details: Some(serde_json::json!({"field": "email"})),
    };
    let json = serde_json::to_string(&with_details).unwrap();
    assert!(json.contains("details"));
    let back: ApiError = serde_json::from_str(&json).unwrap();
    assert!(back.details.is_some());

    let no_details = ApiError {
        code: ApiErrorCode::NotFound,
        message: "Not found".into(),
        details: None,
    };
    let json = serde_json::to_string(&no_details).unwrap();
    assert!(!json.contains("details"));
}

#[test]
fn resource_action_as_str() {
    assert_eq!(Resource::Repository.as_str(), "repository");
    assert_eq!(Resource::PipelineVariable.as_str(), "pipeline_variable");
    assert_eq!(Action::ForcePush.as_str(), "force_push");
    assert_eq!(Action::DownloadArtifact.as_str(), "download_artifact");
}

#[test]
fn permission_check_builder() {
    let allowed = PermissionCheck::allowed(Resource::Repository, Action::Read);
    assert!(allowed.allowed);
    assert!(allowed.reason.is_none());

    let denied = PermissionCheck::denied(Resource::Repository, Action::Delete, "No perms");
    assert!(!denied.allowed);
    assert_eq!(denied.reason.as_deref(), Some("No perms"));
}

#[test]
fn repo_response_roundtrip() {
    let repo = RepoResponse {
        id: RepoId::nil(),
        name: "my-repo".into(),
        full_name: "org/my-repo".into(),
        description: Some("A test repo".into()),
        visibility: Visibility::Public,
        owner_id: UserId::nil(),
        org_id: Some(1),
        default_branch: "main".into(),
        is_fork: false,
        parent_repo_id: None,
        ssh_clone_url: None,
        http_clone_url: Some("https://example.com/repo.git".into()),
        created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap(),
    };
    let json = serde_json::to_string(&repo).unwrap();
    let back: RepoResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "my-repo");
    assert_eq!(back.visibility, Visibility::Public);
}

#[test]
fn create_repo_request_deserialize() {
    let json =
        r#"{"name":"test-repo","description":"Desc","visibility":"public","initialize":true}"#;
    let req: CreateRepoRequest = test_json(json);
    assert_eq!(req.name, "test-repo");
    assert_eq!(req.visibility, Some(Visibility::Public));
    assert_eq!(req.initialize, Some(true));
}

#[test]
fn user_response_roundtrip() {
    let user = UserResponse {
        id: UserId::nil(),
        username: "alice".into(),
        email: "alice@test.com".into(),
        display_name: Some("Alice Smith".into()),
        bio: None,
        role: UserRole::Admin,
        avatar_url: None,
        location: None,
        website: None,
        created_at: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
    };
    let json = serde_json::to_string(&user).unwrap();
    let back: UserResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.username, "alice");
    assert_eq!(back.role, UserRole::Admin);
}

#[test]
fn org_response_roundtrip() {
    let org = OrgResponse {
        id: OrgId::nil(),
        name: "my-org".into(),
        display_name: Some("My Org".into()),
        description: Some("An org".into()),
        visibility: Visibility::Internal,
        owner_id: UserId::nil(),
        member_count: 10,
        repo_count: 5,
        created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
    };
    let json = serde_json::to_string(&org).unwrap();
    let back: OrgResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "my-org");
    assert_eq!(back.member_count, 10);
}

#[test]
fn create_update_org_requests() {
    let create_json = r#"{"name":"new-org","visibility":"private"}"#;
    let create: CreateOrgRequest = test_json(create_json);
    assert_eq!(create.name, "new-org");

    let update_json = r#"{"description":"Updated desc"}"#;
    let update: UpdateOrgRequest = test_json(update_json);
    assert_eq!(update.description.as_deref(), Some("Updated desc"));
    assert!(update.visibility.is_none());
}
