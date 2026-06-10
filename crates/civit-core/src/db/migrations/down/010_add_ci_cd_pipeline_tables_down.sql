-- CivitForge Phase 9: CI/CD Pipeline Backend (Down)
-- Migration 009 Down

DROP TABLE IF EXISTS pipeline_run_steps CASCADE;
DROP TABLE IF EXISTS pipeline_run_jobs CASCADE;
DROP TABLE IF EXISTS pipeline_runs CASCADE;
DROP TABLE IF EXISTS pipeline_job_steps CASCADE;
DROP TABLE IF EXISTS pipeline_jobs CASCADE;
DROP TABLE IF EXISTS pipeline_definitions CASCADE;
DROP TABLE IF EXISTS runners CASCADE;
