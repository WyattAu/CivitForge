CivitForge services must be stateless; no request-local state stored in-memory that isn't shared across instances. All state lives in external stores (PostgreSQL, Redis, S3).
- `db_pool_active_connections`; connections currently in use
- `db_pool_idle_connections`; connections in the pool
- `db_pool_wait_time_seconds`; time waiting for a connection
- `db_pool_checkout_errors`; failed connection acquisitions
- `cache_hit_ratio`; target > 90%
- `cache_miss_rate`; target < 10%
- `cache_eviction_count`; memory pressure indicator
- `redis_memory_used_bytes`; Redis memory usage
- `redis_connected_clients`; Redis connection count