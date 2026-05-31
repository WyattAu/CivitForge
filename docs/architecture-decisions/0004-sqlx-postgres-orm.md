# ADR-0004: sqlx with PostgreSQL (No ORM)

## Status

Accepted

## Context

The application needs database access with compile-time query verification and async support.

## Decision

Use `sqlx` directly with PostgreSQL, without an ORM layer.

## Considerations

- `sqlx` provides compile-time SQL verification via `query!` macros
- Async-native, built on tokio
- PostgreSQL is the primary database for relational data
- Direct SQL gives full control over query optimization
- No ORM abstraction layer reduces complexity and cognitive overhead
- Migrations managed via `sqlx migrate`
- Repository pattern wraps sqlx for clean domain layer boundaries

## Alternatives Considered

- **Diesel**: Compile-time query builder but sync-first with async support as add-on
- **SeaORM**: Full ORM, adds abstraction layer we don't need

## Consequences

- Raw SQL queries in repository layer
- Compile-time checked queries where possible
- Manual migration files in `migrations/`
- Connection pooling via `sqlx::postgres::PgPoolOptions`
