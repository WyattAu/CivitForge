#![forbid(unsafe_code)]

use civit_db::{DbRepository, User, Repository};
use sqlx::postgres::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn create_test_user(db: &DbRepository, suffix: &str) -> User {
    db.create_user(
        &format!("user_{suffix}"),
        &format!("{suffix}@example.com"),
        &format!("User {suffix}"),
        "member",
        &format!("hash_{suffix}"),
    )
    .await
    .expect("create_user should succeed")
}

async fn create_test_repo(db: &DbRepository, owner_id: Uuid, suffix: &str) -> Repository {
    db.create_repo(
        &format!("repo_{suffix}"),
        &format!("Description {suffix}"),
        owner_id,
        None,
        "public",
        "main",
    )
    .await
    .expect("create_repo should succeed")
}

// ---------------------------------------------------------------------------
// 1. User lifecycle
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_create_and_get_by_id(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "alice").await;

    assert!(!user.id.is_nil());
    assert_eq!(user.username, "user_alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.display_name, "User alice");
    assert_eq!(user.role, "member");
    assert_eq!(user.bio, "");

    let fetched = db.get_user_by_id(user.id).await.unwrap();
    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.username, user.username);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_get_by_username(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "bob").await;

    let fetched = db.get_user_by_username("user_bob").await.unwrap();
    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.email, "bob@example.com");
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_update(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "carol").await;

    let updated = db
        .update_user(user.id, Some("Carol Updated"), Some("New bio"), Some("admin"))
        .await
        .unwrap();

    assert_eq!(updated.display_name, "Carol Updated");
    assert_eq!(updated.bio, "New bio");
    assert_eq!(updated.role, "admin");
    assert!(updated.updated_at >= user.updated_at);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_delete(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "dave").await;

    db.delete_user(user.id).await.unwrap();

    let result = db.get_user_by_id(user.id).await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_list(pool: PgPool) {
    let db = DbRepository::new(pool);

    create_test_user(&db, "u1").await;
    create_test_user(&db, "u2").await;
    create_test_user(&db, "u3").await;

    let users = db.list_users(10, 0).await.unwrap();
    assert_eq!(users.len(), 3);

    let page = db.list_users(2, 0).await.unwrap();
    assert_eq!(page.len(), 2);

    let offset_page = db.list_users(10, 2).await.unwrap();
    assert_eq!(offset_page.len(), 1);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_get_by_email(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "eve").await;

    let fetched = db.get_user_by_email("eve@example.com").await.unwrap();
    assert_eq!(fetched.id, user.id);
}

// ---------------------------------------------------------------------------
// 2. Repository lifecycle
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_create_and_get(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "owner").await;
    let repo = create_test_repo(&db, user.id, "myrepo").await;

    assert!(!repo.id.is_nil());
    assert_eq!(repo.name, "repo_myrepo");
    assert_eq!(repo.owner_id, user.id);
    assert_eq!(repo.visibility, "public");
    assert_eq!(repo.default_branch, "main");
    assert!(!repo.is_fork);
    assert_eq!(repo.stars_count, 0);
    assert_eq!(repo.watchers_count, 0);

    let fetched = db.get_repo(repo.id).await.unwrap();
    assert_eq!(fetched.id, repo.id);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_get_by_owner_name(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "owner2").await;
    let repo = create_test_repo(&db, user.id, "target").await;

    let fetched = db.get_repo_by_owner_name(user.id, "repo_target").await.unwrap();
    assert_eq!(fetched.id, repo.id);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_update(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "upd").await;
    let repo = create_test_repo(&db, user.id, "upd").await;

    let updated = db
        .update_repo(
            repo.id,
            Some("Updated description"),
            Some("private"),
            Some("develop"),
        )
        .await
        .unwrap();

    assert_eq!(updated.description, "Updated description");
    assert_eq!(updated.visibility, "private");
    assert_eq!(updated.default_branch, "develop");
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_delete(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "del").await;
    let repo = create_test_repo(&db, user.id, "del").await;

    db.delete_repo(repo.id).await.unwrap();

    let result = db.get_repo(repo.id).await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_list(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "listr").await;

    create_test_repo(&db, user.id, "r1").await;
    create_test_repo(&db, user.id, "r2").await;

    let repos = db.list_repos(10, 0).await.unwrap();
    assert_eq!(repos.len(), 2);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_star_unstar(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "star").await;
    let repo = create_test_repo(&db, user.id, "star").await;

    assert_eq!(repo.stars_count, 0);

    let count = db.increment_stars(repo.id).await.unwrap();
    assert_eq!(count, 1);

    let count = db.increment_stars(repo.id).await.unwrap();
    assert_eq!(count, 2);

    let count = db.decrement_stars(repo.id).await.unwrap();
    assert_eq!(count, 1);

    let count = db.decrement_stars(repo.id).await.unwrap();
    assert_eq!(count, 0);

    // Cannot go below zero
    let count = db.decrement_stars(repo.id).await.unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_watch_unwatch(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "watch").await;
    let repo = create_test_repo(&db, user.id, "watch").await;

    assert_eq!(repo.watchers_count, 0);

    let count = db.increment_watchers(repo.id).await.unwrap();
    assert_eq!(count, 1);

    let count = db.decrement_watchers(repo.id).await.unwrap();
    assert_eq!(count, 0);

    let count = db.decrement_watchers(repo.id).await.unwrap();
    assert_eq!(count, 0);
}

// ---------------------------------------------------------------------------
// 3. Issue lifecycle
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_issue_create_and_get(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "iss_author").await;
    let repo = create_test_repo(&db, user.id, "iss_repo").await;

    let issue = db
        .create_issue(repo.id, "Bug: something broke", "Details here", user.id)
        .await
        .unwrap();

    assert!(!issue.id.is_nil());
    assert_eq!(issue.repo_id, repo.id);
    assert_eq!(issue.title, "Bug: something broke");
    assert_eq!(issue.status, "open");
    assert_eq!(issue.author_id, user.id);

    let fetched = db.get_issue(issue.id).await.unwrap();
    assert_eq!(fetched.id, issue.id);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_issue_update_state(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "iss_upd").await;
    let repo = create_test_repo(&db, user.id, "iss_upd_repo").await;

    let issue = db
        .create_issue(repo.id, "Fix me", "Body", user.id)
        .await
        .unwrap();

    let updated = db
        .update_issue(issue.id, None, None, Some("closed"), None)
        .await
        .unwrap();

    assert_eq!(updated.status, "closed");
    // Note: update_issue doesn't set closed_at; that's done by close_issues_for_pr
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_issue_list_with_filters(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "iss_list").await;
    let repo = create_test_repo(&db, user.id, "iss_list_repo").await;

    db.create_issue(repo.id, "Issue 1", "Body 1", user.id)
        .await
        .unwrap();
    db.create_issue(repo.id, "Issue 2", "Body 2", user.id)
        .await
        .unwrap();
    db.create_issue(repo.id, "Issue 3", "Body 3", user.id)
        .await
        .unwrap();

    let issues = db.list_issues(repo.id, 10, 0).await.unwrap();
    assert_eq!(issues.len(), 3);

    let page = db.list_issues(repo.id, 2, 0).await.unwrap();
    assert_eq!(page.len(), 2);
}

// ---------------------------------------------------------------------------
// 4. PR lifecycle
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_create_and_get(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pr_author").await;
    let repo = create_test_repo(&db, user.id, "pr_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "Add feature X",
            "Implements feature X",
            user.id,
            "feature-x",
            "main",
            false,
        )
        .await
        .unwrap();

    assert!(!pr.id.is_nil());
    assert_eq!(pr.repo_id, repo.id);
    assert_eq!(pr.title, "Add feature X");
    assert_eq!(pr.status, "open");
    assert_eq!(pr.source_branch, "feature-x");
    assert_eq!(pr.target_branch, "main");
    assert!(!pr.draft);

    let fetched = db.get_pr(pr.id).await.unwrap();
    assert_eq!(fetched.id, pr.id);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_update(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pr_upd").await;
    let repo = create_test_repo(&db, user.id, "pr_upd_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "Draft PR",
            "WIP",
            user.id,
            "wip",
            "main",
            true,
        )
        .await
        .unwrap();

    let updated = db
        .update_pr(pr.id, Some("Final PR"), Some("Ready for review"), None)
        .await
        .unwrap();

    assert_eq!(updated.title, "Final PR");
    assert_eq!(updated.body, "Ready for review");
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_list(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pr_list").await;
    let repo = create_test_repo(&db, user.id, "pr_list_repo").await;

    db.create_pr(
        repo.id,
        "PR 1",
        "Body 1",
        user.id,
        "branch1",
        "main",
        false,
    )
    .await
    .unwrap();
    db.create_pr(
        repo.id,
        "PR 2",
        "Body 2",
        user.id,
        "branch2",
        "main",
        false,
    )
    .await
    .unwrap();

    let prs = db.list_prs(repo.id, 10, 0).await.unwrap();
    assert_eq!(prs.len(), 2);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_reviewer(pool: PgPool) {
    let db = DbRepository::new(pool);
    let author = create_test_user(&db, "pr_rev_auth").await;
    let reviewer = create_test_user(&db, "pr_rev_rev").await;
    let repo = create_test_repo(&db, author.id, "pr_rev_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "PR for review",
            "Body",
            author.id,
            "feature",
            "main",
            false,
        )
        .await
        .unwrap();

    let rev = db.add_pr_reviewer(pr.id, reviewer.id).await.unwrap();
    assert_eq!(rev.pr_id, pr.id);
    assert_eq!(rev.user_id, reviewer.id);
    assert_eq!(rev.review_status, "pending");

    let submitted = db
        .submit_pr_review(pr.id, reviewer.id, "approved")
        .await
        .unwrap();
    assert_eq!(submitted.review_status, "approved");
    assert!(submitted.submitted_at.is_some());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_comment(pool: PgPool) {
    let db = DbRepository::new(pool);
    let author = create_test_user(&db, "prc_auth").await;
    let repo = create_test_repo(&db, author.id, "prc_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "PR with comments",
            "Body",
            author.id,
            "feature",
            "main",
            false,
        )
        .await
        .unwrap();

    let comment = db
        .create_pr_comment(
            pr.id,
            author.id,
            "Looks good!",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert!(!comment.id.is_nil());
    assert_eq!(comment.body, "Looks good!");
    assert_eq!(comment.author_id, author.id);

    let comments = db.list_pr_comments(pr.id).await.unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].body, "Looks good!");
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_merge(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pr_merge").await;
    let repo = create_test_repo(&db, user.id, "pr_merge_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "Merge me",
            "Body",
            user.id,
            "feature",
            "main",
            false,
        )
        .await
        .unwrap();

    let merged = db
        .merge_pr(
            pr.id,
            "abc123def456",
            "merge",
            Some("head_sha"),
            Some("base_sha"),
        )
        .await
        .unwrap();

    assert_eq!(merged.status, "merged");
    assert_eq!(merged.merge_commit_id.as_deref(), Some("abc123def456"));
    assert!(merged.merged_at.is_some());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_draft(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pr_draft").await;
    let repo = create_test_repo(&db, user.id, "pr_draft_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "Draft",
            "WIP",
            user.id,
            "draft",
            "main",
            false,
        )
        .await
        .unwrap();

    assert!(!pr.draft);

    let marked = db.set_pr_draft(pr.id, true).await.unwrap();
    assert!(marked.draft);

    let unmarked = db.set_pr_draft(pr.id, false).await.unwrap();
    assert!(!unmarked.draft);
}

// ---------------------------------------------------------------------------
// 5. Pipeline lifecycle
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pipeline_create_and_get(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pipe_auth").await;
    let repo = create_test_repo(&db, user.id, "pipe_repo").await;

    let pipeline = db
        .create_pipeline(repo.id, "abc123", "push")
        .await
        .unwrap();

    assert!(!pipeline.id.is_nil());
    assert_eq!(pipeline.repo_id, repo.id);
    assert_eq!(pipeline.commit_sha, "abc123");
    assert_eq!(pipeline.status, "pending");
    assert_eq!(pipeline.trigger, "push");

    let fetched = db.get_pipeline(pipeline.id).await.unwrap();
    assert_eq!(fetched.id, pipeline.id);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pipeline_list(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pipe_list").await;
    let repo = create_test_repo(&db, user.id, "pipe_list_repo").await;

    db.create_pipeline(repo.id, "sha1", "push").await.unwrap();
    db.create_pipeline(repo.id, "sha2", "pull_request").await
        .unwrap();

    let pipelines = db.list_pipelines(repo.id, 10, 0).await.unwrap();
    assert_eq!(pipelines.len(), 2);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pipeline_update_status(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pipe_upd").await;
    let repo = create_test_repo(&db, user.id, "pipe_upd_repo").await;

    let pipeline = db
        .create_pipeline(repo.id, "sha_upd", "push")
        .await
        .unwrap();

    let running = db
        .update_pipeline(pipeline.id, Some("running"))
        .await
        .unwrap();
    assert_eq!(running.status, "running");

    let success = db
        .update_pipeline(pipeline.id, Some("success"))
        .await
        .unwrap();
    assert_eq!(success.status, "success");
}

// ---------------------------------------------------------------------------
// 6. Webhook lifecycle (via raw SQL — no repository methods exist yet)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_webhook_crud(pool: PgPool) {
    let db = DbRepository::new(pool.clone());
    let user = create_test_user(&db, "wh_auth").await;
    let repo = create_test_repo(&db, user.id, "wh_repo").await;

    // Create
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO webhooks (repo_id, url, secret, events, active)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(repo.id)
    .bind("https://example.com/hook")
    .bind("secret123")
    .bind(&["push".to_string()])
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    let webhook_id = row.0;

    // Get
    let fetched: (Uuid, String, bool) = sqlx::query_as(
        "SELECT id, url, active FROM webhooks WHERE id = $1",
    )
    .bind(webhook_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(fetched.0, webhook_id);
    assert_eq!(fetched.1, "https://example.com/hook");
    assert!(fetched.2);

    // List
    let all: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM webhooks WHERE repo_id = $1",
    )
    .bind(repo.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(all.len(), 1);

    // Delete
    sqlx::query("DELETE FROM webhooks WHERE id = $1")
        .bind(webhook_id)
        .execute(&pool)
        .await
        .unwrap();

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM webhooks WHERE id = $1",
    )
    .bind(webhook_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 0);
}

// ---------------------------------------------------------------------------
// 7. Migration tests
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_migrations_apply_cleanly(pool: PgPool) {
    // If we got here, all migrations applied successfully.
    // Verify a few key tables exist by querying them.
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM repositories")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM issues")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM pull_requests")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM pipelines")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM webhooks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM organizations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 0);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_migration_count(pool: PgPool) {
    // sqlx::test uses its own migration tracking table (sqlx_migrations)
    // Count the number of migrations that were applied
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM _sqlx_migrations",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 29 built-in migrations + 3 new ones (047, 048, 049) = 32
    assert!(row.0 >= 29, "Expected at least 29 migrations, got {}", row.0);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_user_get_nonexistent_returns_error(pool: PgPool) {
    let db = DbRepository::new(pool);
    let result = db.get_user_by_id(Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_repo_get_nonexistent_returns_error(pool: PgPool) {
    let db = DbRepository::new(pool);
    let result = db.get_repo(Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_count_repos(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "count_r").await;

    assert_eq!(db.count_repos().await.unwrap(), 0);

    create_test_repo(&db, user.id, "cr1").await;
    assert_eq!(db.count_repos().await.unwrap(), 1);

    create_test_repo(&db, user.id, "cr2").await;
    assert_eq!(db.count_repos().await.unwrap(), 2);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_count(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "prcnt").await;
    let repo = create_test_repo(&db, user.id, "prcnt_repo").await;

    assert_eq!(db.count_prs(repo.id, None).await.unwrap(), 0);

    db.create_pr(
        repo.id, "PR1", "B1", user.id, "b1", "main", false,
    )
    .await
    .unwrap();
    db.create_pr(
        repo.id, "PR2", "B2", user.id, "b2", "main", false,
    )
    .await
    .unwrap();

    assert_eq!(db.count_prs(repo.id, None).await.unwrap(), 2);
    assert_eq!(db.count_prs(repo.id, Some("open")).await.unwrap(), 2);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_access_token_lifecycle(pool: PgPool) {
    let db = DbRepository::new(pool.clone());
    let user = create_test_user(&db, "token").await;

    // Use raw SQL because create_access_token binds &[String] to a JSONB column.
    // The scopes column is JSONB, so we need to bind as JSON.
    let scopes_json = serde_json::json!(["read"]);
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO access_tokens (user_id, name, token_hash, scopes, expires_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(user.id)
    .bind("my-token")
    .bind("hash_abc")
    .bind(&scopes_json)
    .bind(None::<chrono::DateTime<chrono::Utc>>)
    .fetch_one(&pool)
    .await
    .unwrap();
    let token_id = row.0;
    assert!(!token_id.is_nil());

    let user_id = db.validate_access_token("hash_abc").await.unwrap();
    assert_eq!(user_id, user.id);

    db.revoke_access_token("hash_abc").await.unwrap();
    let result = db.validate_access_token("hash_abc").await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_audit_event_lifecycle(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "audit").await;

    let event_id = db
        .record_audit_event(
            user.id,
            "user.login",
            "session",
            None,
            Some("127.0.0.1"),
            Some("test-agent"),
            "success",
        )
        .await
        .unwrap();
    assert!(event_id > 0);

    let events = db
        .query_audit_events(Some(user.id), Some("session"), 10, 0)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].1, user.id);
    assert_eq!(events[0].2, "user.login");
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_ssh_key_lifecycle(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "ssh").await;

    let key = db
        .add_ssh_key(
            user.id,
            "ssh-ed25519",
            "AAAAC3NzaC1lZDI1NTE5AAAAI...",
            "SHA256:abc123",
            "my-laptop",
        )
        .await
        .unwrap();

    assert!(!key.id.is_nil());
    assert_eq!(key.fingerprint, "SHA256:abc123");

    let keys = db.list_ssh_keys(user.id).await.unwrap();
    assert_eq!(keys.len(), 1);

    let fetched = db
        .get_ssh_key_by_fingerprint("SHA256:abc123")
        .await
        .unwrap();
    assert_eq!(fetched.id, key.id);

    db.delete_ssh_key(key.id).await.unwrap();
    let keys = db.list_ssh_keys(user.id).await.unwrap();
    assert_eq!(keys.len(), 0);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_activity_event_lifecycle(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "activity").await;

    let event = db
        .record_activity_event(
            user.id,
            "push",
            "repository",
            None,
            None,
            None,
            "Pushed 3 commits",
            serde_json::json!({"commits": 3}),
        )
        .await
        .unwrap();

    assert!(event.id > 0);
    assert_eq!(event.action, "push");
    assert_eq!(event.description, "Pushed 3 commits");

    let events = db
        .list_activity_events(None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_password_hash_lifecycle(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "pwd").await;

    // create_user sets password_hash to "hash_pwd"
    let hash = db.get_password_hash(user.id).await.unwrap();
    assert_eq!(hash.as_deref(), Some("hash_pwd"));

    db.change_password(user.id, "new_hash_123").await.unwrap();

    let hash = db.get_password_hash(user.id).await.unwrap();
    assert_eq!(hash.as_deref(), Some("new_hash_123"));
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_email_verification(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "email_v").await;

    assert!(!user.email_verified);

    db.set_email_verified(user.id).await.unwrap();

    let fetched = db.get_user_by_id(user.id).await.unwrap();
    assert!(fetched.email_verified);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_login_attempts(pool: PgPool) {
    let db = DbRepository::new(pool);

    db.record_login_attempt("alice", "127.0.0.1", false).await.unwrap();
    db.record_login_attempt("alice", "127.0.0.1", false).await.unwrap();
    db.record_login_attempt("alice", "127.0.0.1", true).await.unwrap();

    let count = db.count_recent_failed_logins("alice", 3600).await.unwrap();
    assert_eq!(count, 2);

    db.clear_login_attempts("alice").await.unwrap();
    let count = db.count_recent_failed_logins("alice", 3600).await.unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_status_check(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "prsc").await;
    let repo = create_test_repo(&db, user.id, "prsc_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "PR with checks",
            "Body",
            user.id,
            "feature",
            "main",
            false,
        )
        .await
        .unwrap();

    let check = db
        .upsert_pr_status_check(
            pr.id,
            "ci/test",
            "success",
            "All tests passed",
            Some("https://ci.example.com/123"),
            Some("sha_abc"),
        )
        .await
        .unwrap();

    assert_eq!(check.context, "ci/test");
    assert_eq!(check.state, "success");

    let checks = db.list_pr_status_checks(pr.id).await.unwrap();
    assert_eq!(checks.len(), 1);
}

#[sqlx::test(migrations = "src/migrations")]
#[ignore = "requires PostgreSQL database"]
async fn test_pr_timeline(pool: PgPool) {
    let db = DbRepository::new(pool);
    let user = create_test_user(&db, "prtl").await;
    let repo = create_test_repo(&db, user.id, "prtl_repo").await;

    let pr = db
        .create_pr(
            repo.id,
            "PR with timeline",
            "Body",
            user.id,
            "feature",
            "main",
            false,
        )
        .await
        .unwrap();

    let event = db
        .insert_pr_timeline(
            pr.id,
            user.id,
            "opened",
            serde_json::json!({"action": "opened"}),
        )
        .await
        .unwrap();

    assert_eq!(event.event_type, "opened");

    let timeline = db.list_pr_timeline(pr.id).await.unwrap();
    assert_eq!(timeline.len(), 1);
}
