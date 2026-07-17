#![forbid(unsafe_code)]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;

// ── Git operations ────────────────────────────────────────────────────────────

fn bench_git_init_bare(c: &mut Criterion) {
    c.bench_function("git_init_bare", |b| {
        b.iter(|| {
            let dir = tempfile::tempdir().unwrap();
            let s = civit_git::GitService::new(dir.path().to_path_buf());
            black_box(s.init_bare("bench-org", &format!("repo-{}", uuid::Uuid::new_v4())))
        });
    });
}

fn bench_git_open_repo(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage);
    svc.init_bare("bench-org", "open-test").unwrap();
    let path = svc.repo_path("bench-org", "open-test");

    c.bench_function("git_open_repo", |b| {
        b.iter(|| black_box(gix::open(&path).unwrap()));
    });
}

fn bench_git_list_commits(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "commit-test").unwrap();

    let repo_path = storage.join("bench-org").join("commit-test.git");
    create_test_repo_with_commits(&repo_path, 20);

    c.bench_function("git_list_commits_20", |b| {
        b.iter(|| black_box(svc.list_commits("bench-org", "commit-test", 20).unwrap()));
    });

    c.bench_function("git_list_commits_100", |b| {
        b.iter(|| black_box(svc.list_commits("bench-org", "commit-test", 100).unwrap()));
    });
}

fn bench_git_read_tree(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "tree-test").unwrap();

    let repo_path = storage.join("bench-org").join("tree-test.git");
    create_test_repo_with_tree(&repo_path);

    c.bench_function("git_read_tree_root", |b| {
        b.iter(|| black_box(civit_git::read_tree(&repo_path, "HEAD", "").unwrap()));
    });
}

fn bench_git_diff(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let storage = tmp.path().join("repos");
    std::fs::create_dir_all(&storage).unwrap();
    let svc = civit_git::GitService::new(storage.clone());
    svc.init_bare("bench-org", "diff-test").unwrap();

    let repo_path = storage.join("bench-org").join("diff-test.git");
    let (base, head) = create_test_repo_with_diff(&repo_path);

    c.bench_function("git_diff_5_files", |b| {
        b.iter(|| black_box(civit_git::generate_diff(&repo_path, &base, &head).unwrap()));
    });
}

// ── JWT authentication ────────────────────────────────────────────────────────

fn bench_jwt_generate(c: &mut Criterion) {
    let secret = "bench-secret-key-32bytes-minimum!";
    let svc = civit_auth::jwt::JwtService::new(secret, 24).unwrap();

    c.bench_function("jwt_generate_token", |b| {
        b.iter(|| {
            black_box(
                svc.generate_token("user-1", "alice", "admin", Some("org-1"))
                    .unwrap(),
            )
        });
    });
}

fn bench_jwt_validate(c: &mut Criterion) {
    let secret = "bench-secret-key-32bytes-minimum!";
    let svc = civit_auth::jwt::JwtService::new(secret, 24).unwrap();
    let token = svc
        .generate_token("user-1", "alice", "admin", Some("org-1"))
        .unwrap();

    c.bench_function("jwt_validate_token", |b| {
        b.iter(|| black_box(svc.validate_token(&token).unwrap()));
    });
}

fn bench_jwt_extract_bearer(c: &mut Criterion) {
    let header = "Bearer eyJhbGciOiJIUzI1NiJ9.test.token";

    c.bench_function("jwt_extract_bearer", |b| {
        b.iter(|| black_box(civit_auth::jwt::JwtService::extract_bearer(header)));
    });
}

// ── Password hashing ─────────────────────────────────────────────────────────

fn bench_password_hash(c: &mut Criterion) {
    c.bench_function("password_hash", |b| {
        b.iter(|| black_box(civit_auth::password::hash_password("benchmark-password").unwrap()));
    });
}

fn bench_password_verify(c: &mut Criterion) {
    let hash = civit_auth::password::hash_password("benchmark-password").unwrap();

    c.bench_function("password_verify_correct", |b| {
        b.iter(|| black_box(civit_auth::password::verify_password("benchmark-password", &hash)));
    });

    c.bench_function("password_verify_wrong", |b| {
        b.iter(|| black_box(civit_auth::password::verify_password("wrong-password", &hash)));
    });
}

// ── Password policy validation ───────────────────────────────────────────────

fn bench_password_policy_check(c: &mut Criterion) {
    let policy = civit_auth::password::PasswordPolicy::default();
    let password = "StrongP@ssw0rd!";

    c.bench_function("password_policy_check", |b| {
        b.iter(|| {
            black_box(civit_auth::password::validate_password_policy(
                password, &policy,
            ))
        });
    });
}

// ── Storage chunking ─────────────────────────────────────────────────────────

fn bench_storage_chunking(c: &mut Criterion) {
    let chunker =
        civit_core::storage::chunking::ContentDefinedChunker::new(Default::default());

    let mut group = c.benchmark_group("storage_chunk");
    for size in [64 * 1024, 256 * 1024, 1024 * 1024] {
        let data = vec![0xABu8; size];
        group.bench_with_input(
            BenchmarkId::new("chunk_write", format!("{size}_bytes")),
            &data,
            |b, data| {
                b.iter(|| black_box(chunker.chunk(data)));
            },
        );
    }
    group.finish();
}

fn bench_storage_chunk_assemble(c: &mut Criterion) {
    let chunker =
        civit_core::storage::chunking::ContentDefinedChunker::new(Default::default());
    let data = vec![0xCDu8; 256 * 1024];
    let chunks = chunker.chunk(&data);

    c.bench_function("storage_chunk_assemble", |b| {
        b.iter(|| black_box(civit_core::storage::chunking::ContentDefinedChunker::reconstruct(
            &chunks,
        )));
    });
}

// ── Event bus ─────────────────────────────────────────────────────────────────

fn bench_event_bus_publish(c: &mut Criterion) {
    let bus = civit_core::events::bus::EventBus::new(1024);

    c.bench_function("event_bus_publish", |b| {
        b.iter(|| {
            black_box(bus.publish(
                "bench",
                civit_core::events::model::Event::new(
                    civit_core::events::model::EventCategory::System,
                    civit_core::events::model::EventPayload::SystemEvent {
                        level: civit_core::events::model::SystemLevel::Info,
                        message: "bench event".into(),
                    },
                    "bench".into(),
                ),
            ));
        });
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn git_cmd(work: &std::path::Path, args: &[&str]) -> String {
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

fn create_test_repo_with_commits(repo_path: &PathBuf, count: usize) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work.to_str().unwrap()],
    );
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    for i in 0..count {
        std::fs::write(work.join(format!("file-{i}.txt")), format!("content-{i}\n")).unwrap();
        git_cmd(&work, &["add", "."]);
        git_cmd(&work, &["commit", "-m", &format!("commit {i}")]);
    }

    git_cmd(&work, &["push", "origin", "main"]);
}

fn create_test_repo_with_tree(repo_path: &PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work.to_str().unwrap()],
    );
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    for dir in &["src", "tests", "docs"] {
        std::fs::create_dir_all(work.join(dir)).unwrap();
    }
    for file in &["src/main.rs", "src/lib.rs", "tests/test.rs", "docs/readme.md"] {
        std::fs::write(work.join(file), "// placeholder\n").unwrap();
    }

    git_cmd(&work, &["add", "."]);
    git_cmd(&work, &["commit", "-m", "add tree structure"]);
    git_cmd(&work, &["push", "origin", "main"]);
}

fn create_test_repo_with_diff(repo_path: &PathBuf) -> (String, String) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    git_cmd(
        tmp.path(),
        &["clone", repo_path.to_str().unwrap(), work.to_str().unwrap()],
    );
    git_cmd(&work, &["config", "user.name", "bench"]);
    git_cmd(&work, &["config", "user.email", "bench@test.com"]);

    for i in 0..5 {
        std::fs::write(work.join(format!("file-{i}.txt")), "initial\n").unwrap();
    }
    git_cmd(&work, &["add", "."]);
    git_cmd(&work, &["commit", "-m", "initial files"]);

    let base = git_cmd(&work, &["rev-parse", "HEAD"])
        .trim()
        .to_string();

    for i in 0..5 {
        std::fs::write(
            work.join(format!("file-{i}.txt")),
            format!("modified-{i}\n"),
        )
        .unwrap();
    }
    git_cmd(&work, &["add", "."]);
    git_cmd(&work, &["commit", "-m", "modify all files"]);

    let head = git_cmd(&work, &["rev-parse", "HEAD"])
        .trim()
        .to_string();

    git_cmd(&work, &["push", "origin", "main"]);

    (base, head)
}

criterion_group!(
    benches,
    bench_git_init_bare,
    bench_git_open_repo,
    bench_git_list_commits,
    bench_git_read_tree,
    bench_git_diff,
    bench_jwt_generate,
    bench_jwt_validate,
    bench_jwt_extract_bearer,
    bench_password_hash,
    bench_password_verify,
    bench_password_policy_check,
    bench_storage_chunking,
    bench_storage_chunk_assemble,
    bench_event_bus_publish,
);
criterion_main!(benches);
