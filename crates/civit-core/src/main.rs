#![forbid(unsafe_code)]

use anyhow::Result;
use std::sync::Arc;

use civit_core::{api::create_router, config::AppConfig};
use shutdown_kit::shutdown_signal;
use std::net::SocketAddr;
use tracing::{error, info};

/// Splits a migration script into statements.
///
/// Comment-aware, and that is not optional. A `;` inside a `--` comment used
/// to split mid-comment, leaving a fragment that no longer began with `--`,
/// so Postgres parsed the prose as SQL: migration 640 shipped with
/// "lifecycle category; sets the staleness deadline" in a comment and the
/// server died with `syntax error at or near "sets"`. A migration that
/// applies cleanly under `psql -f` can still break here, so the runner — not
/// the migration author — has to be correct.
///
/// Handles `--` line comments, nested-free `/* */` block comments, single
/// quotes (with `''` escapes), dollar-quoted strings, and `;` as the only
/// statement terminator.
#[cfg(test)]
mod sql_split_tests {
    use super::split_sql_statements;

    fn parts(sql: &str) -> Vec<String> {
        split_sql_statements(sql)
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// The regression that shipped: a `;` inside a `--` comment used to split
    /// mid-comment, leaving a fragment that began with prose and was parsed
    /// as SQL. Migration 640 died with `syntax error at or near "sets"`.
    #[test]
    fn semicolon_inside_a_line_comment_does_not_split() {
        let sql = "-- kind sets the deadline; sets the staleness clock\nCREATE TABLE a (id INT);";
        let p = parts(sql);
        assert_eq!(p.len(), 1, "comment semicolon must not split: {p:?}");
        assert!(p[0].contains("CREATE TABLE a"));
    }

    #[test]
    fn block_comment_with_semicolons_is_ignored() {
        let sql = "/* step one; step two; three */ CREATE TABLE b (id INT); SELECT 1;";
        let p = parts(sql);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[0].contains("CREATE TABLE b"));
        assert_eq!(p[1], "SELECT 1");
    }

    /// A `;` inside a quoted literal is data, not a terminator.
    #[test]
    fn semicolon_inside_a_string_literal_does_not_split() {
        let sql = "INSERT INTO t VALUES ('a;b'); SELECT 2;";
        let p = parts(sql);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[0].contains("'a;b'"));
    }

    #[test]
    fn escaped_quote_inside_literal_is_handled() {
        let sql = "INSERT INTO t VALUES ('it''s; fine'); SELECT 3;";
        let p = parts(sql);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[0].contains("it''s; fine"));
    }

    #[test]
    fn dollar_quoted_body_is_untouched() {
        let sql = "CREATE FUNCTION f() RETURNS int AS $$ BEGIN; RETURN 1; END; $$ LANGUAGE plpgsql; SELECT 4;";
        let p = parts(sql);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[0].contains("RETURN 1; END;"));
    }

    #[test]
    fn tagged_dollar_quote_is_untouched() {
        let sql = "CREATE FUNCTION g() RETURNS int AS $body$ SELECT 1; $body$ LANGUAGE sql; SELECT 5;";
        let p = parts(sql);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[1] == "SELECT 5");
    }

    #[test]
    fn plain_statements_still_split() {
        let p = parts("SELECT 1; SELECT 2;\nSELECT 3");
        assert_eq!(p.len(), 3, "{p:?}");
    }

    /// Every shipped migration must survive the runner's splitter.
    #[test]
    fn every_shipped_migration_splits_into_executable_fragments() {
        use civit_db::migrations::MigrationManager;
        let mgr = MigrationManager::new();
        let mut checked = 0usize;
        for m in mgr.all() {
            let fragments = parts(&m.up_sql);
            for f in &fragments {
                // A fragment must either be comment-only or start with
                // something SQL-ish. A fragment starting with prose is the
                // exact failure mode that shipped.
                let body = f
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("--"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let body = body.trim();
                if body.is_empty() {
                    continue;
                }
                let first = body.split_whitespace().next().unwrap_or("");
                assert!(
                    first.chars().next().is_some_and(|c| c.is_ascii_alphabetic()),
                    "migration {} ({}) produced a fragment starting with {:?}",
                    m.version,
                    m.name,
                    first
                );
                checked += 1;
            }
        }
        assert!(checked > 100, "expected the whole migration set, got {checked}");
    }
}

fn split_sql_statements(sql: &str) -> Vec<&str> {
    let bytes = sql.as_bytes();
    let mut statements: Vec<&str> = Vec::new();
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_single_quote = false;
    let mut start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i];
        let next = bytes.get(i + 1).copied();

        if in_line_comment {
            if c == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && next == Some(b'/') {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_single_quote {
            // '' inside a quoted literal is an escaped quote, not a terminator.
            if c == b'\'' && next == Some(b'\'') {
                i += 2;
                continue;
            }
            if c == b'\'' {
                in_single_quote = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'-' if next == Some(b'-') => {
                in_line_comment = true;
                i += 2;
            }
            b'/' if next == Some(b'*') => {
                in_block_comment = true;
                i += 2;
            }
            b'\'' => {
                in_single_quote = true;
                i += 1;
            }
            b'$' => {
                // Dollar-quoted region: $tag$ ... $tag$, where the empty tag
                // is $$. The whole region is skipped in one jump rather than
                // tracked as a toggle: a toggle never looks for the closing
                // tag, so everything after `$$` was swallowed and no
                // statement after a dollar-quoted function body ever split.
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'$' {
                    let tag = &sql[i..=j];
                    match sql[j + 1..].find(tag) {
                        Some(offset) => {
                            i = j + 1 + offset + tag.len();
                            continue;
                        }
                        // Unterminated quote: the rest of the script is the
                        // body, so no `;` inside it can split anything.
                        None => {
                            i = bytes.len();
                            continue;
                        }
                    }
                }
                // A bare `$` that opens no tag is just a character.
                i += 1;
            }
            b';' => {
                statements.push(&sql[start..i]);
                start = i + 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    if start < sql.len() {
        statements.push(&sql[start..]);
    }
    statements
}

#[tokio::main]
async fn main() -> Result<()> {
    let debug_mode = std::env::args().any(|arg| arg == "--debug");

    // ADR-0006 Phase 4: otelkit subscriber (real OTLP export when
    // OTEL_EXPORTER_OTLP_ENDPOINT is set; compact human logs otherwise).
    let log_level = if std::env::var("RUST_LOG").is_ok() {
        std::env::var("RUST_LOG").unwrap()
    } else if debug_mode {
        "civit_core=debug,tower_http=debug".into()
    } else {
        "civit_core=info,tower_http=debug".into()
    };
    let telemetry = otelkit::TelemetryConfig::new("civitforge")
        .service_version(env!("CARGO_PKG_VERSION"))
        .log_level(log_level)
        .log_format(otelkit::LogFormat::Text);
    // Guard must live for the process lifetime: flushes spans on drop.
    let _telemetry_guard = otelkit::init(telemetry)?;

    // W3C TraceContext propagator for inbound `traceparent`. otelkit 2.1
    // installs one itself; setting it here is idempotent and keeps the
    // behavior explicit if that ever changes.
    otelkit::otel::opentelemetry::global::set_text_map_propagator(
        otelkit::otel::opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    let mut config = AppConfig::from_env()?;
    if debug_mode {
        config.debug_mode = true;
    }

    info!("connecting to database");
    let db_pool = civit_core::db::pool_from_config(&config).await?;
    let pool = db_pool.pool().clone();

    let migration_mgr = civit_core::db::migrations::MigrationManager::new();
    let current_version: i64 =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .map(|r| r.0)
            .unwrap_or(0);

    let pending = migration_mgr.get_pending(current_version);
    if !pending.is_empty() {
        info!(
            current = current_version,
            pending = pending.len(),
            "running pending migrations"
        );
        for migration in &pending {
            info!(
                version = migration.version,
                name = %migration.name,
                "applying migration"
            );
            for stmt in split_sql_statements(&migration.up_sql) {
                let s = stmt.trim();
                if !s.is_empty() {
                    sqlx::query(sqlx::AssertSqlSafe(s.to_string()))
                        .execute(&pool)
                        .await?;
                }
            }
            sqlx::query(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES ($1, $2, NOW())",
            )
            .bind(migration.version)
            .bind(&migration.name)
            .execute(&pool)
            .await?;
            info!(version = migration.version, "migration applied");
        }
        info!("all migrations applied");
    } else {
        info!(current = current_version, "database schema is up to date");
    }

    // create_router consumes the pool, so the controller gets its own clone
    // of the same handle rather than a second connection pool.
    let router = create_router(config.clone(), pool.clone())?;

    // Health-gated rollout controller (ADR-0008). Drives in-flight flag
    // rollouts from the rolling health window: promote on green, roll back on
    // red, and hold when there is not enough evidence either way.
    if config.rollout_controller_enabled() {
        let rollout_db = Arc::new(civit_core::db::DbRepository::new(pool.clone()));
        let rollout_config = civit_core::rollout_controller::RolloutControllerConfig::default();
        let tick = rollout_config.tick;
        let controller =
            civit_core::rollout_controller::RolloutController::new(rollout_db, rollout_config);
        tokio::spawn(async move {
            // Stagger the first tick so it does not race startup migrations.
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            loop {
                let window = civit_telemetry::global_health_window();
                match controller.tick_once(window).await {
                    Ok(observations) => {
                        for obs in observations {
                            if obs.decision != flag_kit::gate::Decision::Hold {
                                info!(
                                    flag = %obs.flag_name,
                                    decision = %obs.decision.as_str(),
                                    reason = %obs.reason,
                                    from = obs.percentage_before,
                                    to = obs.percentage_after,
                                    error_rate = ?obs.error_rate,
                                    p99_ms = ?obs.latency_p99_ms,
                                    "health-gated rollout"
                                );
                            }
                        }
                    }
                    Err(e) => error!(error = %e, "rollout controller tick failed"),
                }
                tokio::time::sleep(tick).await;
            }
        });
        info!("health-gated rollout controller started");
    } else {
        info!("health-gated rollout controller disabled (CIVIT_ROLLOUT_CONTROLLER=false)");
    }

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| {
            anyhow::anyhow!(
                "invalid bind address '{}:{}': {e}",
                config.host,
                config.port
            )
        })?;

    if config.tls_enabled() {
        let cert_path = config.tls_cert_path.as_ref().expect("operation should succeed");
        let key_path = config.tls_key_path.as_ref().expect("operation should succeed");

        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| anyhow::anyhow!("failed to load TLS certificate/key: {e}"))?;

        info!("CivitForge API listening on {} (TLS)", addr);
        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(router.into_make_service())
            .await?;
    } else {
        info!("CivitForge API listening on {} (HTTP)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    }

    info!("server shutdown complete");
    Ok(())
}
