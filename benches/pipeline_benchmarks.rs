use criterion::{black_box, criterion_group, criterion_main, Criterion};
use civit_pipeline::{expand_matrix, parse_pipeline, validate_pipeline};

const SMALL_YAML: &str = r#"
version: '1'
jobs:
  - name: build
    steps:
      - name: test
        run: ["cargo test"]
"#;

const MEDIUM_YAML: &str = r#"
version: '1'
jobs:
  - name: lint
    steps:
      - name: clippy
        run: ["cargo clippy -- -D warnings"]
  - name: test
    needs: [lint]
    steps:
      - name: unit
        run: ["cargo test --lib"]
  - name: build
    needs: [test]
    steps:
      - name: compile
        run: ["cargo build --release"]
"#;

const LARGE_YAML: &str = r#"
version: '1'
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
jobs:
  - name: check-format
    runs-on:
      labels: ["ubuntu-latest"]
    steps:
      - name: checkout
        checkout: {}
      - name: fmt
        run: ["cargo fmt --check"]
  - name: clippy
    runs-on:
      labels: ["ubuntu-latest"]
    steps:
      - name: checkout
        checkout: {}
      - name: clippy
        run: ["cargo clippy --all-targets -- -D warnings"]
  - name: test-unit
    needs: [check-format, clippy]
    runs-on:
      labels: ["ubuntu-latest"]
    steps:
      - name: checkout
        checkout: {}
      - name: test
        run: ["cargo test --lib"]
      - name: coverage
        run: ["cargo tarpaulin --out xml"]
  - name: test-integration
    needs: [check-format, clippy]
    runs-on:
      labels: ["ubuntu-latest"]
    services:
      - name: postgres
        image: postgres:16
        ports:
          - port: 5432
            protocol: tcp
        env:
          - name: POSTGRES_DB
            value: "civit_test"
      - name: redis
        image: redis:7
        ports:
          - port: 6379
            protocol: tcp
    steps:
      - name: checkout
        checkout: {}
      - name: migrate
        run: ["sqlx migrate run"]
      - name: test
        run: ["cargo test --test integration"]
  - name: build-release
    needs: [test-unit, test-integration]
    runs-on:
      labels: ["ubuntu-latest"]
    steps:
      - name: checkout
        checkout: {}
      - name: build
        run: ["cargo build --release"]
      - name: artifact
        artifact:
          name: release-binary
          path: ["target/release/civitforge"]
          retention: "30d"
"#;

fn bench_yaml_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("yaml_parse");
    group.bench_function("small", |b| {
        b.iter(|| black_box(parse_pipeline(SMALL_YAML).unwrap()));
    });
    group.bench_function("medium", |b| {
        b.iter(|| black_box(parse_pipeline(MEDIUM_YAML).unwrap()));
    });
    group.bench_function("large", |b| {
        b.iter(|| black_box(parse_pipeline(LARGE_YAML).unwrap()));
    });
    group.finish();
}

fn bench_matrix_expand(c: &mut Criterion) {
    let yaml_2x2 = r#"
version: '1'
jobs:
  - name: build
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, nightly]
    runs-on:
      labels: ["${{ matrix.os }}"]
    steps:
      - name: test
        run: ["cargo +${{ matrix.rust }} test"]
"#;

    let yaml_3x3 = r#"
version: '1'
jobs:
  - name: build
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly, beta]
        profile: [dev, release]
    runs-on:
      labels: ["${{ matrix.os }}"]
    steps:
      - name: test
        run: ["cargo +${{ matrix.rust }} test --profile ${{ matrix.profile }}"]
"#;

    let pipeline_2x2 = parse_pipeline(yaml_2x2).unwrap();
    let pipeline_3x3 = parse_pipeline(yaml_3x3).unwrap();

    let mut group = c.benchmark_group("matrix_expand");
    group.bench_function("2x2", |b| {
        b.iter(|| black_box(expand_matrix(&pipeline_2x2).unwrap()));
    });
    group.bench_function("3x3", |b| {
        b.iter(|| black_box(expand_matrix(&pipeline_3x3).unwrap()));
    });
    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    let pipeline = parse_pipeline(LARGE_YAML).unwrap();
    c.bench_function("validate_large_pipeline", |b| {
        b.iter(|| validate_pipeline(&pipeline).unwrap());
    });
}

criterion_group!(
    benches,
    bench_yaml_parse,
    bench_matrix_expand,
    bench_validation,
);
criterion_main!(benches);
