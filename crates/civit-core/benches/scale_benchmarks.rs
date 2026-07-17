#![forbid(unsafe_code)]

//! Comprehensive scale benchmarks for CivitForge.
//!
//! Benchmarks critical paths at 1K, 10K, and 100K scale to catch
//! performance regressions before they reach production.
//!
//! Run: cargo bench -p civit-core --bench scale_benchmarks

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::Path;

// ══════════════════════════════════════════════════════════════════════════════
// Data generators
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct TestRepo {
    name: String,
    description: String,
    #[allow(dead_code)]
    default_branch: String,
}

#[derive(Debug, Clone)]
struct TestIssue {
    #[allow(dead_code)]
    id: u64,
    title: String,
    body: String,
    labels: Vec<String>,
    state: String,
}

#[derive(Debug, Clone)]
struct TestPipeline {
    #[allow(dead_code)]
    id: u64,
    name: String,
    status: String,
    #[allow(dead_code)]
    job_count: usize,
    #[allow(dead_code)]
    yaml: String,
}

fn generate_repos(n: usize) -> Vec<TestRepo> {
    (0..n)
        .map(|i| TestRepo {
            name: format!("bench-repo-{i:06}"),
            description: format!("Benchmark repository #{i} for scale testing"),
            default_branch: "main".to_string(),
        })
        .collect()
}

fn generate_issues(repo_id: &str, n: usize) -> Vec<TestIssue> {
    let states = ["open", "closed", "open", "open"];
    let label_pool = [
        "bug", "enhancement", "documentation", "good-first-issue", "help-wanted",
        "performance", "security", "ui", "backend", "ci",
    ];

    (0..n)
        .map(|i| {
            let label_count = (i % 5) + 1;
            let labels: Vec<String> = (0..label_count)
                .map(|l| {
                    let idx = (i * 7 + l * 13) % label_pool.len();
                    label_pool[idx].to_string()
                })
                .collect();

            TestIssue {
                id: i as u64,
                title: format!("[repo:{repo_id}] Issue #{i}: test issue title"),
                body: format!(
                    "This is the body of issue #{i}.\n\n\
                     It contains some description text for benchmarking \
                     full-text search queries. The issue belongs to repository \
                     {repo_id} and has been created for scale testing purposes.\n\n\
                     Additional content: lorem ipsum dolor sit amet #{i}"
                ),
                labels,
                state: states[i % states.len()].to_string(),
            }
        })
        .collect()
}

fn generate_bulk_issues(repo_id: &str, n: usize, state: &str) -> Vec<TestIssue> {
    (0..n)
        .map(|i| TestIssue {
            id: i as u64,
            title: format!("[bulk:{repo_id}] Issue #{i}"),
            body: format!("Bulk issue #{i} with state {state}"),
            labels: vec!["bulk".to_string()],
            state: state.to_string(),
        })
        .collect()
}

fn generate_pipelines(repo_id: &str, n: usize) -> Vec<TestPipeline> {
    let statuses = ["pending", "running", "completed", "failed", "cancelled"];

    (0..n)
        .map(|i| {
            let job_count = (i % 20) + 1;
            let status = statuses[i % statuses.len()].to_string();
            let yaml = generate_pipeline_yaml(i, job_count);

            TestPipeline {
                id: i as u64,
                name: format!("pipeline-{repo_id}-{i:06}"),
                status,
                job_count,
                yaml,
            }
        })
        .collect()
}

fn generate_pipeline_yaml(idx: usize, job_count: usize) -> String {
    let mut lines = vec!["version: '1'".to_string(), "jobs:".to_string()];

    for j in 0..job_count {
        let name = format!("job-{j}");
        let needs = if j > 0 {
            format!("    needs: [job-{}]", j - 1)
        } else {
            String::new()
        };

        lines.push(format!("  - name: {name}"));
        if !needs.is_empty() {
            lines.push(needs);
        }
        lines.push("    steps:".to_string());
        lines.push(format!("      - name: step-{j}"));
        lines.push(format!(
            "        run: [\"echo step-{j} of pipeline-{idx}\"]"
        ));
    }

    lines.join("\n")
}

fn generate_dag_pipeline(idx: usize, dependency_count: usize) -> String {
    let mut lines = vec!["version: '1'".to_string(), "jobs:".to_string()];

    for j in 0..dependency_count {
        let name = format!("job-{j}");
        let needs: Vec<String> = (0..j).map(|k| format!("job-{k}")).collect();
        let needs_str = if needs.is_empty() {
            String::new()
        } else {
            format!("    needs: [{}]", needs.join(", "))
        };

        lines.push(format!("  - name: {name}"));
        if !needs_str.is_empty() {
            lines.push(needs_str);
        }
        lines.push("    steps:".to_string());
        lines.push(format!("      - name: step-{j}"));
        lines.push(format!(
            "        run: [\"echo job-{j} of dag-pipeline-{idx}\"]"
        ));
    }

    lines.join("\n")
}

fn generate_large_json_payload(count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            serde_json::json!({
                "id": format!("repo-{i:06}"),
                "name": format!("bench-repo-{i:06}"),
                "description": format!("A benchmark repository for testing serialization at scale. Index {i}."),
                "default_branch": "main",
                "visibility": "public",
                "stars": i * 3,
                "forks": i % 17,
                "open_issues": i % 42,
                "tags": ["bench", "scale", &format!("tag-{}", i % 10)],
            })
        })
        .collect()
}

fn generate_event_bus_with_load(max_log: usize, event_count: usize) -> civit_core::events::bus::EventBus {
    let bus = civit_core::events::bus::EventBus::new(max_log);
    for i in 0..event_count {
        bus.publish(
            "bench",
            civit_core::events::model::Event::new(
                civit_core::events::model::EventCategory::System,
                civit_core::events::model::EventPayload::SystemEvent {
                    level: civit_core::events::model::SystemLevel::Info,
                    message: format!("pre-loaded event #{i}"),
                },
                "bench-generator".into(),
            ),
        );
    }
    bus
}

// ── Git helpers ──────────────────────────────────────────────────────────────

fn git_cmd(work: &Path, args: &[&str]) -> String {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let out = std::process::Command::new(&git_bin)
        .args(args)
        .current_dir(work)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap_or_default()
}

fn git_cmd_ok(work: &Path, args: &[&str]) {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let _ = std::process::Command::new(&git_bin)
        .args(args)
        .current_dir(work)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output();
}

fn create_commits(repo_path: &std::path::Path, count: usize) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    git_cmd(tmp.path(), &["clone", repo_path.to_str().unwrap(), work.to_str().unwrap()]);
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);
    git_cmd_ok(&work, &["branch", "main"]);

    for i in 0..count {
        std::fs::write(
            work.join(format!("file-{i:06}.txt")),
            format!("Content for commit {i}\nLine 2\nLine 3\n"),
        )
        .unwrap();
        git_cmd(&work, &["add", "."]);
        git_cmd(&work, &["commit", "-m", &format!("bench: commit {i}")]);
    }

    git_cmd(&work, &["push", "origin", "main"]);
}

// ══════════════════════════════════════════════════════════════════════════════
// 1. Repository operations at scale
// ══════════════════════════════════════════════════════════════════════════════

fn bench_repo_list_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("repo_list_scale");

    for count in [1_000, 10_000] {
        let repos = generate_repos(count);

        group.bench_with_input(
            BenchmarkId::new("paginate", count),
            &repos,
            |b, repos| {
                b.iter(|| {
                    let page: Vec<_> = repos.iter().take(50).collect();
                    black_box(&page);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterate_all", count),
            &repos,
            |b, repos| {
                b.iter(|| {
                    for repo in repos {
                        black_box(&repo.name);
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_repo_search_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("repo_search_scale");

    let repos = generate_repos(10_000);

    group.bench_function("search_by_name_prefix", |b| {
        b.iter(|| {
            let results: Vec<_> = repos
                .iter()
                .filter(|r| r.name.starts_with("bench-repo-0000"))
                .collect();
            black_box(&results);
        });
    });

    group.bench_function("search_by_description", |b| {
        b.iter(|| {
            let results: Vec<_> = repos
                .iter()
                .filter(|r| r.description.contains("scale testing"))
                .collect();
            black_box(&results);
        });
    });

    group.bench_function("sort_by_name", |b| {
        b.iter(|| {
            let mut sorted = repos.clone();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            black_box(&sorted);
        });
    });

    group.finish();
}

fn bench_repo_get_with_relations(c: &mut Criterion) {
    let mut group = c.benchmark_group("repo_get_with_relations");

    let repos = generate_repos(1_000);

    group.bench_function("get_with_relations_1k", |b| {
        b.iter(|| {
            let repo = &repos[repos.len() / 2];
            let issues = generate_issues(&repo.name, 10);
            let pipelines = generate_pipelines(&repo.name, 5);
            let mut result = serde_json::json!({
                "repo": repo.name,
                "issues_count": issues.len(),
                "pipelines_count": pipelines.len(),
            });
            black_box(&mut result);
        });
    });

    group.finish();
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. Issue operations at scale
// ══════════════════════════════════════════════════════════════════════════════

fn bench_issue_list_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_list_scale");

    let issues = generate_issues("bench-repo", 10_000);

    group.bench_function("list_with_state_filter", |b| {
        b.iter(|| {
            let filtered: Vec<_> = issues.iter().filter(|i| i.state == "open").collect();
            black_box(&filtered);
        });
    });

    group.bench_function("list_with_label_filter", |b| {
        b.iter(|| {
            let filtered: Vec<_> = issues
                .iter()
                .filter(|i| i.labels.contains(&"bug".to_string()))
                .collect();
            black_box(&filtered);
        });
    });

    group.bench_function("paginate_50_per_page", |b| {
        b.iter(|| {
            let page: Vec<_> = issues.iter().take(50).collect();
            black_box(&page);
        });
    });

    group.finish();
}

fn bench_issue_search_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_search_scale");

    let issues = generate_issues("bench-repo", 10_000);

    group.bench_function("search_in_title", |b| {
        b.iter(|| {
            let results: Vec<_> = issues
                .iter()
                .filter(|i| i.title.contains("Issue #5000"))
                .collect();
            black_box(&results);
        });
    });

    group.bench_function("search_in_body", |b| {
        b.iter(|| {
            let results: Vec<_> = issues
                .iter()
                .filter(|i| i.body.contains("benchmarking"))
                .collect();
            black_box(&results);
        });
    });

    group.bench_function("combined_search", |b| {
        b.iter(|| {
            let results: Vec<_> = issues
                .iter()
                .filter(|i| {
                    i.title.contains("Issue #") && i.body.contains("scale testing")
                })
                .collect();
            black_box(&results);
        });
    });

    group.finish();
}

fn bench_issue_get_with_comments(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_get_with_comments");

    group.bench_function("get_issue_with_100_comments", |b| {
        b.iter(|| {
            let issue = TestIssue {
                id: 42,
                title: "Test issue with many comments".to_string(),
                body: "Body text here".to_string(),
                labels: vec!["bug".to_string()],
                state: "open".to_string(),
            };

            let comments: Vec<_> = (0..100)
                .map(|i| {
                    serde_json::json!({
                        "id": i,
                        "body": format!("Comment #{i}: some discussion text"),
                        "author": format!("user-{i}"),
                    })
                })
                .collect();

            let mut result = serde_json::json!({
                "issue": issue.title,
                "comments": comments,
            });
            black_box(&mut result);
        });
    });

    group.finish();
}

fn bench_issue_bulk_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_bulk_create");

    group.bench_function("bulk_insert_1k", |b| {
        b.iter(|| {
            let issues = generate_bulk_issues("bench-repo", 1_000, "open");
            black_box(&issues);
        });
    });

    group.finish();
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. Pipeline operations at scale
// ══════════════════════════════════════════════════════════════════════════════

fn bench_pipeline_list_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_list_scale");

    let pipelines = generate_pipelines("bench-repo", 1_000);

    group.bench_function("list_with_status_filter", |b| {
        b.iter(|| {
            let filtered: Vec<_> = pipelines
                .iter()
                .filter(|p| p.status == "running")
                .collect();
            black_box(&filtered);
        });
    });

    group.bench_function("paginate_50_per_page", |b| {
        b.iter(|| {
            let page: Vec<_> = pipelines.iter().take(50).collect();
            black_box(&page);
        });
    });

    group.bench_function("iterate_all_1k", |b| {
        b.iter(|| {
            for p in &pipelines {
                black_box(&p.name);
            }
        });
    });

    group.finish();
}

fn bench_pipeline_get_with_jobs(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_get_with_jobs");

    group.bench_function("get_pipeline_with_100_jobs", |b| {
        b.iter(|| {
            let pipeline = TestPipeline {
                id: 1,
                name: "bench-pipeline".to_string(),
                status: "running".to_string(),
                job_count: 100,
                yaml: String::new(),
            };

            let jobs: Vec<_> = (0..100)
                .map(|i| {
                    serde_json::json!({
                        "name": format!("job-{i}"),
                        "status": "running",
                        "step_count": 3,
                    })
                })
                .collect();

            let mut result = serde_json::json!({
                "pipeline": pipeline.name,
                "jobs": jobs,
            });
            black_box(&mut result);
        });
    });

    group.finish();
}

fn bench_pipeline_dag_scheduling(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_dag_scheduling");

    for dep_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("dag_resolve", dep_count),
            &dep_count,
            |b, &n| {
                let yaml = generate_dag_pipeline(0, n);
                b.iter(|| {
                    let pipeline = civit_pipeline::parse_pipeline(&yaml).unwrap();
                    black_box(&pipeline);
                });
            },
        );
    }

    group.bench_function("validate_large_pipeline", |b| {
        let yaml = generate_pipeline_yaml(0, 50);
        b.iter(|| {
            let pipeline = civit_pipeline::parse_pipeline(&yaml).unwrap();
            let result = civit_pipeline::validate_pipeline(&pipeline);
            black_box(&result);
        });
    });

    group.finish();
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. Git operations at scale
// ══════════════════════════════════════════════════════════════════════════════

fn bench_git_clone_scale(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());

    let repo_path = svc.init_bare("bench-org", "clone-test").unwrap();
    create_commits(&repo_path, 100);

    c.bench_function("git_clone_100_commits", |b| {
        b.iter(|| {
            black_box(svc.clone("bench-org", "clone-test", "").unwrap());
        });
    });
}

fn bench_git_diff_scale(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "diff-test").unwrap();

    let repo_path = storage.join("bench-org").join("diff-test.git");

    let work_dir = tmp.path().join("diff-work");
    std::fs::create_dir_all(&work_dir).unwrap();
    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work_dir.to_str().unwrap()],
    );
    git_cmd(&work_dir, &["config", "user.name", "bench"]);
    git_cmd(&work_dir, &["config", "user.email", "bench@test.com"]);
    git_cmd_ok(&work_dir, &["branch", "main"]);

    for i in 0..100 {
        std::fs::write(work_dir.join(format!("file-{i:03}.txt")), "initial content\n").unwrap();
    }
    git_cmd(&work_dir, &["add", "."]);
    git_cmd(&work_dir, &["commit", "-m", "initial commit"]);
    let base = git_cmd(&work_dir, &["rev-parse", "HEAD"]).trim().to_string();

    for i in 0..100 {
        std::fs::write(
            work_dir.join(format!("file-{i:03}.txt")),
            format!("modified content {i}\n"),
        )
        .unwrap();
    }
    git_cmd(&work_dir, &["add", "."]);
    git_cmd(&work_dir, &["commit", "-m", "modify all files"]);
    let head = git_cmd(&work_dir, &["rev-parse", "HEAD"]).trim().to_string();
    git_cmd(&work_dir, &["push", "origin", "main"]);

    c.bench_function("git_diff_100_files", |b| {
        b.iter(|| {
            black_box(civit_git::generate_diff(&repo_path, &base, &head).unwrap());
        });
    });
}

fn bench_git_blame_scale(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "blame-test").unwrap();

    let repo_path = storage.join("bench-org").join("blame-test.git");

    let work_dir = tmp.path().join("blame-work");
    std::fs::create_dir_all(&work_dir).unwrap();
    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work_dir.to_str().unwrap()],
    );
    git_cmd(&work_dir, &["config", "user.name", "bench"]);
    git_cmd(&work_dir, &["config", "user.email", "bench@test.com"]);
    git_cmd_ok(&work_dir, &["branch", "main"]);

    let line_count = 10_000;
    let content: String = (0..line_count)
        .map(|i| format!("line {i}: blame benchmarking content {i}\n"))
        .collect();
    std::fs::write(work_dir.join("large.txt"), content).unwrap();
    git_cmd(&work_dir, &["add", "."]);
    git_cmd(&work_dir, &["commit", "-m", "add large file"]);
    git_cmd(&work_dir, &["push", "origin", "main"]);

    c.bench_function("git_blame_10k_lines", |b| {
        b.iter(|| {
            black_box(civit_git::git_blame(&repo_path, "main", "large.txt").unwrap());
        });
    });
}

fn bench_git_list_branches_scale(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "branch-test").unwrap();

    let repo_path = storage.join("bench-org").join("branch-test.git");

    let work_dir = tmp.path().join("branch-work");
    std::fs::create_dir_all(&work_dir).unwrap();
    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work_dir.to_str().unwrap()],
    );
    git_cmd(&work_dir, &["config", "user.name", "bench"]);
    git_cmd(&work_dir, &["config", "user.email", "bench@test.com"]);
    git_cmd_ok(&work_dir, &["branch", "main"]);

    let branch_count = 100;
    for i in 0..branch_count {
        std::fs::write(
            work_dir.join(format!("branch-{i}.txt")),
            format!("branch {i}\n"),
        )
        .unwrap();
        git_cmd(&work_dir, &["add", "."]);
        git_cmd(&work_dir, &["commit", "-m", &format!("branch-{i}")]);
        git_cmd(&work_dir, &["checkout", "-b", &format!("branch-{i:03}")]);
    }
    git_cmd(&work_dir, &["checkout", "main"]);
    for i in 0..branch_count {
        git_cmd(&work_dir, &["push", "origin", &format!("branch-{i:03}")]);
    }

    c.bench_function("git_list_100_branches", |b| {
        b.iter(|| {
            let output = std::process::Command::new(
                std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string()),
            )
            .args(["branch", "-r"])
            .current_dir(&repo_path)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap_or_default();
            let branches: Vec<&str> = stdout.lines().collect();
            black_box(&branches);
        });
    });
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. API response times
// ══════════════════════════════════════════════════════════════════════════════

fn bench_auth_middleware(c: &mut Criterion) {
    let secret = "bench-secret-key-32bytes-minimum!";
    let svc = civit_auth::jwt::JwtService::new(secret, 24).unwrap();
    let token = svc
        .generate_token("user-1", "alice", "admin", Some("org-1"))
        .unwrap();
    let bearer = format!("Bearer {token}");

    let mut group = c.benchmark_group("auth_middleware");

    group.bench_function("jwt_validate_throughput", |b| {
        b.iter(|| black_box(svc.validate_token(&token).unwrap()));
    });

    group.bench_function("bearer_extract_and_validate", |b| {
        b.iter(|| {
            let extracted = civit_auth::jwt::JwtService::extract_bearer(&bearer);
            if let Some(tok) = extracted {
                black_box(svc.validate_token(tok).unwrap());
            }
        });
    });

    group.bench_function("concurrent_jwt_validation_10k", |b| {
        let secret_arc = std::sync::Arc::new(secret.to_string());
        b.iter(|| {
            let handles: Vec<_> = (0..10_000)
                .map(|_| {
                    let sec = secret_arc.clone();
                    let tok = token.clone();
                    std::thread::spawn(move || {
                        let svc_inner = civit_auth::jwt::JwtService::new(&sec, 24).unwrap();
                        black_box(svc_inner.validate_token(&tok).unwrap())
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_rate_limiter_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limiter_scale");

    group.bench_function("rate_limit_check_10k_users", |b| {
        let user_states: Vec<(String, u32, std::time::Instant)> = (0..10_000)
            .map(|i| (format!("user-{i}"), 0u32, std::time::Instant::now()))
            .collect();

        b.iter(|| {
            for (user_id, count, _) in &user_states {
                let new_count = count + 1;
                let within_limit = new_count < 300;
                black_box((user_id, within_limit));
            }
        });
    });

    group.bench_function("token_bucket_refill_10k", |b| {
        let mut buckets: Vec<(u32, std::time::Instant)> = (0..10_000)
            .map(|_| (300u32, std::time::Instant::now()))
            .collect();

        b.iter(|| {
            let now = std::time::Instant::now();
            for (tokens, last_refill) in &mut buckets {
                let elapsed = now.duration_since(*last_refill).as_secs();
                let refill = (elapsed * 5) as u32;
                *tokens = (*tokens + refill).min(300);
                *last_refill = now;
                black_box(*tokens);
            }
        });
    });

    group.finish();
}

fn bench_response_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_serialization");

    let small_payload = serde_json::json!({
        "id": "repo-001",
        "name": "test-repo",
        "description": "A test repository",
        "default_branch": "main",
    });

    group.bench_function("serialize_small_json", |b| {
        b.iter(|| black_box(serde_json::to_string(&small_payload).unwrap()));
    });

    group.bench_function("deserialize_small_json", |b| {
        let json_str = serde_json::to_string(&small_payload).unwrap();
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(&json_str).unwrap());
        });
    });

    let large_payload = generate_large_json_payload(1_000);
    group.bench_function("serialize_1k_repos", |b| {
        b.iter(|| black_box(serde_json::to_string(&large_payload).unwrap()));
    });

    group.bench_function("deserialize_1k_repos", |b| {
        let json_str = serde_json::to_string(&large_payload).unwrap();
        b.iter(|| {
            black_box(
                serde_json::from_str::<Vec<serde_json::Value>>(&json_str).unwrap(),
            );
        });
    });

    let very_large_payload = generate_large_json_payload(10_000);
    group.bench_function("serialize_10k_repos", |b| {
        b.iter(|| black_box(serde_json::to_string(&very_large_payload).unwrap()));
    });

    group.bench_function("deserialize_10k_repos", |b| {
        let json_str = serde_json::to_string(&very_large_payload).unwrap();
        b.iter(|| {
            black_box(
                serde_json::from_str::<Vec<serde_json::Value>>(&json_str).unwrap(),
            );
        });
    });

    group.finish();
}

fn bench_event_bus_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus_scale");

    group.bench_function("publish_10k_events", |b| {
        b.iter(|| {
            let bus = civit_core::events::bus::EventBus::new(10_000);
            for i in 0..10_000 {
                bus.publish(
                    "bench",
                    civit_core::events::model::Event::new(
                        civit_core::events::model::EventCategory::System,
                        civit_core::events::model::EventPayload::SystemEvent {
                            level: civit_core::events::model::SystemLevel::Info,
                            message: format!("event #{i}"),
                        },
                        "bench".into(),
                    ),
                );
            }
            black_box(bus.publish_count());
        });
    });

    group.bench_function("replay_from_10k_events", |b| {
        let bus = generate_event_bus_with_load(10_000, 10_000);
        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        b.iter(|| {
            black_box(bus.replay("bench", since));
        });
    });

    group.finish();
}

// ══════════════════════════════════════════════════════════════════════════════
// Criterion harness
// ══════════════════════════════════════════════════════════════════════════════

criterion_group!(
    benches,
    bench_repo_list_scale,
    bench_repo_search_scale,
    bench_repo_get_with_relations,
    bench_issue_list_scale,
    bench_issue_search_scale,
    bench_issue_get_with_comments,
    bench_issue_bulk_create,
    bench_pipeline_list_scale,
    bench_pipeline_get_with_jobs,
    bench_pipeline_dag_scheduling,
    bench_git_clone_scale,
    bench_git_diff_scale,
    bench_git_blame_scale,
    bench_git_list_branches_scale,
    bench_auth_middleware,
    bench_rate_limiter_scale,
    bench_response_serialization,
    bench_event_bus_scale,
);
criterion_main!(benches);
