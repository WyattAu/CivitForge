-- Migration 031 down: Remove PR tracking tables and columns

DROP TABLE IF EXISTS pr_status_checks;
DROP TABLE IF EXISTS pr_timeline;
DROP TABLE IF EXISTS pr_reviewers;
DROP TABLE IF EXISTS pr_assignees;
DROP TABLE IF EXISTS pr_labels;
DROP TABLE IF EXISTS pr_comments;

ALTER TABLE pull_requests DROP COLUMN IF EXISTS merge_strategy;
ALTER TABLE pull_requests DROP COLUMN IF EXISTS base_commit_sha;
ALTER TABLE pull_requests DROP COLUMN IF EXISTS head_commit_sha;
ALTER TABLE pull_requests DROP COLUMN IF EXISTS draft;
