#![forbid(unsafe_code)]

//! Benchmark data generators for scale tests.
//!
//! Provides functions to create test data at various scales (1K–100K)
//! for use in criterion benchmarks.
//!
//! Run: cargo bench -p civit-core --bench generators

use std::path::{Path, PathBuf};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use civit_core::events::bus::EventBus;
use civit_core::events::model::{Event, EventCategory, EventPayload, SystemLevel};
use civit_core::storage::chunking::{ChunkerConfig, ContentDefinedChunker};

// ── Repo generators ──────────────────────────────────────────────────────────

/// Metadata for a generated test repository.
#[derive(Debug, Clone)]
pub struct TestRepo {
    pub name: String,
    pub description: String,
    pub default_branch: String,
    pub storage_path: PathBuf,
}

/// Generate N test repository metadata entries (in-memory, no disk).
pub fn generate_repos(n: usize) -> Vec<TestRepo> {
    (0..n)
        .map(|i| TestRepo {
            name: format!("bench-repo-{i:06}"),
            description: format!("Benchmark repository #{i} for scale testing"),
            default_branch: "main".to_string(),
            storage_path: PathBuf::from(format!("/tmp/bench/repos/{i}.git")),
        })
        .collect()
}

/// Create N bare git repositories on disk under the given base directory.
pub fn create_bare_repos_on_disk(base: &Path, n: usize) -> Vec<PathBuf> {
    std::fs::create_dir_all(base).expect("create repos dir");

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

    (0..n)
        .map(|i| {
            let repo_path = base.join(format!("repo-{i:06}.git"));
            let output = std::process::Command::new(&git_bin)
                .args(["init", "--bare", repo_path.to_str().unwrap()])
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .expect("git init --bare failed");
            assert!(
                output.status.success(),
                "git init --bare failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            repo_path
        })
        .collect()
}

// ── Issue generators ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TestIssue {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub state: String,
}

/// Generate N test issues (in-memory, no DB).
pub fn generate_issues(repo_id: &str, n: usize) -> Vec<TestIssue> {
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

/// Bulk-create N issues with the given state filter (simulates bulk insert).
pub fn generate_bulk_issues(repo_id: &str, n: usize, state: &str) -> Vec<TestIssue> {
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

// ── Pipeline generators ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TestPipeline {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub job_count: usize,
    pub yaml: String,
}

#[derive(Debug, Clone)]
pub struct TestJob {
    pub name: String,
    pub needs: Vec<String>,
    pub step_count: usize,
}

/// Generate N test pipelines with jobs.
pub fn generate_pipelines(repo_id: &str, n: usize) -> Vec<TestPipeline> {
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

/// Generate a pipeline YAML with the given number of jobs forming a chain.
pub fn generate_pipeline_yaml(idx: usize, job_count: usize) -> String {
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
        lines.push(format!(
            "      - name: step-{j}"
        ));
        lines.push(format!(
            "        run: [\"echo step-{j} of pipeline-{idx}\"]"
        ));
    }

    lines.join("\n")
}

/// Generate a pipeline with N interdependent jobs (DAG shape).
pub fn generate_dag_pipeline(idx: usize, dependency_count: usize) -> String {
    let mut lines = vec!["version: '1'".to_string(), "jobs:".to_string()];

    for j in 0..dependency_count {
        let name = format!("job-{j}");

        // Each job depends on all previous jobs (full DAG, not chain)
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

// ── Git commit generators ────────────────────────────────────────────────────

fn git_cmd(work: &Path, args: &[&str]) -> String {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let out = std::process::Command::new(&git_bin)
        .args(args)
        .current_dir(work)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git command failed");
    assert!(
        out.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap_or_default()
}

/// Create N commits in a cloned working copy of a bare repo, then push.
/// Returns (commit_oids, file_names) for later use.
pub fn create_commits(repo_path: &Path, count: usize) -> Vec<String> {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    let bare_str = repo_path.to_str().unwrap();
    git_cmd(tmp.path(), &["clone", bare_str, work.to_str().unwrap()]);
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    // Create a branch if no main branch exists (bare repos have no HEAD)
    let _init_output = std::process::Command::new(
        std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string()),
    )
    .args(["branch", "main"])
    .current_dir(&work)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .output()
    .unwrap();
    // Ignore error if branch already exists

    let mut oids = Vec::with_capacity(count);
    for i in 0..count {
        let filename = format!("file-{i:06}.txt");
        std::fs::write(
            work.join(&filename),
            format!("Content for commit {i}\nLine 2\nLine 3\n"),
        )
        .unwrap();

        git_cmd(&work, &["add", "."]);
        git_cmd(&work, &["commit", "-m", &format!("bench: commit {i}")]);
        let oid = git_cmd(&work, &["rev-parse", "HEAD"]).trim().to_string();
        oids.push(oid);
    }

    git_cmd(&work, &["push", "origin", "main"]);
    oids
}

/// Create a large file (N lines) in a cloned working copy and commit it.
pub fn create_large_file(repo_path: &Path, line_count: usize) -> String {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    let bare_str = repo_path.to_str().unwrap();
    git_cmd(tmp.path(), &["clone", bare_str, work.to_str().unwrap()]);
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    let _ = std::process::Command::new(
        std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string()),
    )
    .args(["branch", "main"])
    .current_dir(&work)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .output();

    // Write a file with N distinct lines
    let content: String = (0..line_count)
        .map(|i| format!("line {i}: some content for blame benchmarking {i}\n"))
        .collect();
    std::fs::write(work.join("large-file.txt"), content).unwrap();

    git_cmd(&work, &["add", "."]);
    git_cmd(&work, &["commit", "-m", "bench: add large file"]);
    git_cmd(&work, &["push", "origin", "main"]);

    git_cmd(&work, &["rev-parse", "HEAD"]).trim().to_string()
}

/// Create N branches in a cloned working copy and push them.
pub fn create_branches(repo_path: &Path, count: usize) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    let bare_str = repo_path.to_str().unwrap();
    git_cmd(tmp.path(), &["clone", bare_str, work.to_str().unwrap()]);
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    let _ = std::process::Command::new(
        std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string()),
    )
    .args(["branch", "main"])
    .current_dir(&work)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .output();

    for i in 0..count {
        let branch_name = format!("branch-{i:03}");
        std::fs::write(work.join(format!("branch-{i}.txt")), format!("branch {i}\n")).unwrap();
        git_cmd(&work, &["add", "."]);
        git_cmd(&work, &["commit", "-m", &format!("branch-{i}")]);
        git_cmd(
            &work,
            &["checkout", "-b", &branch_name],
        );
    }

    // Push all branches
    for i in 0..count {
        let branch_name = format!("branch-{i:03}");
        git_cmd(
            &work,
            &["push", "origin", &branch_name],
        );
    }
}

// ── Storage / chunking generators ────────────────────────────────────────────

/// Generate chunk data of the specified size in bytes.
pub fn generate_chunk_data(size_bytes: usize) -> Vec<u8> {
    let _chunker = ContentDefinedChunker::new(ChunkerConfig::default());
    let data: Vec<u8> = (0..size_bytes).map(|i| (i % 251) as u8).collect();
    data
}

// ── Event bus generators ─────────────────────────────────────────────────────

/// Create an EventBus pre-loaded with N events.
pub fn generate_event_bus_with_load(max_log: usize, event_count: usize) -> EventBus {
    let bus = EventBus::new(max_log);
    for i in 0..event_count {
        bus.publish(
            "bench",
            Event::new(
                EventCategory::System,
                EventPayload::SystemEvent {
                    level: SystemLevel::Info,
                    message: format!("pre-loaded event #{i}"),
                },
                "bench-generator".into(),
            ),
        );
    }
    bus
}

// ── Serialization generators ─────────────────────────────────────────────────

/// Generate a large JSON payload (vector of repo-like objects) for serialization benchmarks.
pub fn generate_large_json_payload(count: usize) -> Vec<serde_json::Value> {
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

// ── Generator throughput benchmarks ───────────────────────────────────────────

fn bench_generate_repos(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_repos");
    for n in [1_000, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(generate_repos(n)));
        });
    }
    group.finish();
}

fn bench_generate_issues(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_issues");
    for n in [1_000, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(generate_issues("bench-repo", n)));
        });
    }
    group.finish();
}

fn bench_generate_pipelines(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_pipelines");
    for n in [1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(generate_pipelines("bench-repo", n)));
        });
    }
    group.finish();
}

fn bench_generate_pipeline_yaml(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_pipeline_yaml");
    for job_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(job_count),
            &job_count,
            |b, &n| {
                b.iter(|| black_box(generate_pipeline_yaml(0, n)));
            },
        );
    }
    group.finish();
}

fn bench_generate_dag_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_dag_pipeline");
    for dep_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(dep_count),
            &dep_count,
            |b, &n| {
                b.iter(|| black_box(generate_dag_pipeline(0, n)));
            },
        );
    }
    group.finish();
}

fn bench_generate_chunk_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_chunk_data");
    for size in [64 * 1024, 256 * 1024, 1024 * 1024] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}_bytes")),
            &size,
            |b, &size| {
                b.iter(|| black_box(generate_chunk_data(size)));
            },
        );
    }
    group.finish();
}

fn bench_generate_large_json_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_large_json_payload");
    for n in [1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(generate_large_json_payload(n)));
        });
    }
    group.finish();
}

fn bench_generate_event_bus(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_event_bus");
    for n in [1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| black_box(generate_event_bus_with_load(n, n)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_generate_repos,
    bench_generate_issues,
    bench_generate_pipelines,
    bench_generate_pipeline_yaml,
    bench_generate_dag_pipeline,
    bench_generate_chunk_data,
    bench_generate_large_json_payload,
    bench_generate_event_bus,
);
criterion_main!(benches);
