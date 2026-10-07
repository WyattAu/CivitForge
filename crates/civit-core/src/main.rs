#![forbid(unsafe_code)]

use anyhow::Result;
use std::sync::Arc;

use civit_core::{api::create_router, config::AppConfig};
use shutdown_kit::shutdown_signal;
use std::net::SocketAddr;
use tracing::{error, info};

fn split_sql_statements(sql: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut in_dollar_quote = false;
    let mut start = 0;
    let mut chars = sql.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '$' {
            let mut tag = String::new();
            tag.push(c);
            while let Some(&(_, next_c)) = chars.peek() {
                if next_c == '$' {
                    tag.push(next_c);
                    chars.next();
                    break;
                } else if next_c.is_alphanumeric() || next_c == '_' {
                    tag.push(next_c);
                    chars.next();
                } else {
                    tag.clear();
                    tag.push(c);
                    break;
                }
            }
            if tag.starts_with("$$") && tag.ends_with("$$") {
                in_dollar_quote = !in_dollar_quote;
            }
        } else if c == ';' && !in_dollar_quote {
            statements.push(&sql[start..i]);
            start = i + 1;
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
