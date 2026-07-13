-- Down migration 091: feature flags
DROP TABLE IF EXISTS feature_flag_events;
DROP TABLE IF EXISTS feature_flags;
