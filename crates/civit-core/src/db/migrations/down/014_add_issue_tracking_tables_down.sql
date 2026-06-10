-- Down migration for 013: Issue Tracking tables
DROP TABLE IF EXISTS issue_reactions;
DROP TABLE IF EXISTS issue_timeline;
DROP TABLE IF EXISTS issue_assignees;
DROP TABLE IF EXISTS milestones;
DROP TABLE IF EXISTS issue_labels;
DROP TABLE IF EXISTS labels;
DROP TABLE IF EXISTS issue_comments;
DROP TABLE IF EXISTS issues;
