---
title: API Reference
description: REST API reference for authentication, repositories, issues, pull requests, pipelines, wiki, search, and webhooks.
---

## Overview

The CivitForge API is served by `civit-core` on the configured `CIVIT_HOST:CIVIT_PORT`
(default `127.0.0.1:8080`). All endpoints are prefixed with `/api/v1/`.

## Authentication

### Register

```
POST /api/v1/auth/register
```

**Request:**

```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "SecurePass123!"
}
```

**Response (201):**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "token": "eyJhbGciOiJIUzI1NiJ9..."
}
```

### Login

```
POST /api/v1/auth/login
```

**Request:**

```json
{
  "username": "alice",
  "password": "SecurePass123!"
}
```

**Response (200):**

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2026-06-20T12:00:00Z"
}
```

### Personal Access Tokens

```
POST /api/v1/tokens
GET  /api/v1/tokens
DELETE /api/v1/tokens/{token_id}
```

Tokens are returned once at creation. Store them securely.

## Repositories

### List repositories

```
GET /api/v1/repos
GET /api/v1/users/{username}/repos
GET /api/v1/orgs/{org_name}/repos
```

### Create repository

```
POST /api/v1/repos
```

**Request:**

```json
{
  "name": "my-project",
  "description": "A new project",
  "visibility": "public",
  "default_branch": "main"
}
```

### Get repository

```
GET /api/v1/repos/{owner}/{name}
```

### Update repository

```
PATCH /api/v1/repos/{owner}/{name}
```

### Delete repository

```
DELETE /api/v1/repos/{owner}/{name}
```

### Archive/unarchive

```
POST /api/v1/repos/{owner}/{name}/archive
POST /api/v1/repos/{owner}/{name}/unarchive
```

### Topics

```
PUT /api/v1/repos/{owner}/{name}/topics
```

**Request:**

```json
{
  "topics": ["rust", "web", "federation"]
}
```

## Issues

### List issues

```
GET /api/v1/repos/{owner}/{name}/issues?status=open&assignee=alice
```

### Create issue

```
POST /api/v1/repos/{owner}/{name}/issues
```

**Request:**

```json
{
  "title": "Bug: login fails on Safari",
  "body": "Steps to reproduce...",
  "labels": ["bug", "priority:high"],
  "assignee": "bob"
}
```

### Get issue

```
GET /api/v1/repos/{owner}/{name}/issues/{number}
```

### Update issue

```
PATCH /api/v1/repos/{owner}/{name}/issues/{number}
```

### Close issue

```
POST /api/v1/repos/{owner}/{name}/issues/{number}/close
```

### Reopen issue

```
POST /api/v1/repos/{owner}/{name}/issues/{number}/reopen
```

### Add comment

```
POST /api/v1/repos/{owner}/{name}/issues/{number}/comments
```

**Request:**

```json
{
  "body": "I can reproduce this on Safari 17.4."
}
```

### List comments

```
GET /api/v1/repos/{owner}/{name}/issues/{number}/comments
```

### Assignees

```
POST /api/v1/repos/{owner}/{name}/issues/{number}/assignees
DELETE /api/v1/repos/{owner}/{name}/issues/{number}/assignees/{username}
```

### Labels

```
PUT /api/v1/repos/{owner}/{name}/issues/{number}/labels
DELETE /api/v1/repos/{owner}/{name}/issues/{number}/labels/{label}
```

## Pull Requests

### List pull requests

```
GET /api/v1/repos/{owner}/{name}/pulls?status=open
```

### Create pull request

```
POST /api/v1/repos/{owner}/{name}/pulls
```

**Request:**

```json
{
  "title": "Fix: resolve Safari login issue",
  "body": "Closes #42",
  "source_branch": "fix/safari-login",
  "target_branch": "main"
}
```

### Get pull request

```
GET /api/v1/repos/{owner}/{name}/pulls/{number}
```

### Update pull request

```
PATCH /api/v1/repos/{owner}/{name}/pulls/{number}
```

### Merge pull request

```
POST /api/v1/repos/{owner}/{name}/pulls/{number}/merge
```

### Close pull request

```
POST /api/v1/repos/{owner}/{name}/pulls/{number}/close
```

### Diff

```
GET /api/v1/repos/{owner}/{name}/pulls/{number}/diff
```

Returns a unified diff.

### Reviews

```
POST /api/v1/repos/{owner}/{name}/pulls/{number}/reviews
```

**Request:**

```json
{
  "status": "approved",
  "body": "LGTM"
}
```

Review statuses: `pending`, `approved`, `changes_requested`, `commented`.

## Pipelines

### List pipelines

```
GET /api/v1/repos/{owner}/{name}/pipelines?status=success
```

### Trigger pipeline

```
POST /api/v1/repos/{owner}/{name}/pipelines
```

**Request:**

```json
{
  "commit_sha": "abc123def456",
  "trigger": "push"
}
```

### Get pipeline

```
GET /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}
```

### Get pipeline steps

```
GET /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}/steps
```

### Get step logs

```
GET /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}/steps/{step_id}/logs
```

### Cancel pipeline

```
POST /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}/cancel
```

### Retry pipeline

```
POST /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}/retry
```

### Pipeline schedules

```
GET /api/v1/repos/{owner}/{name}/schedules
POST /api/v1/repos/{owner}/{name}/schedules
PATCH /api/v1/repos/{owner}/{name}/schedules/{schedule_id}
DELETE /api/v1/repos/{owner}/{name}/schedules/{schedule_id}
```

### Status badge

```
GET /api/v1/repos/{owner}/{name}/pipelines/{pipeline_id}/badge.svg
```

Returns an SVG badge for embedding in README files.

## Wiki

### List pages

```
GET /api/v1/repos/{owner}/{name}/wiki/pages
```

### Get page

```
GET /api/v1/repos/{owner}/{name}/wiki/pages/{page_slug}
```

### Create page

```
POST /api/v1/repos/{owner}/{name}/wiki/pages
```

**Request:**

```json
{
  "slug": "getting-started",
  "title": "Getting Started",
  "content": "# Getting Started\n\nFollow these steps..."
}
```

### Update page

```
PUT /api/v1/repos/{owner}/{name}/wiki/pages/{page_slug}
```

### Delete page

```
DELETE /api/v1/repos/{owner}/{name}/wiki/pages/{page_slug}
```

### Page history

```
GET /api/v1/repos/{owner}/{name}/wiki/pages/{page_slug}/history
```

### Page diff

```
GET /api/v1/repos/{owner}/{name}/wiki/pages/{page_slug}/diff?from={version}&to={version}
```

## Code Search

### Search

```
GET /api/v1/search?q=query&repo={owner}/{name}&language=rust
```

**Response:**

```json
{
  "results": [
    {
      "file": "src/main.rs",
      "line": 42,
      "content": "fn main() {",
      "repo": "owner/project",
      "score": 1.0
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 20
}
```

### Search parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Search query (required) |
| `repo` | string | Filter by `owner/name` |
| `language` | string | Filter by language |
| `path` | string | Filter by file path |
| `page` | int | Page number (default 1) |
| `per_page` | int | Results per page (default 20, max 100) |

## Webhooks

### List webhooks

```
GET /api/v1/repos/{owner}/{name}/webhooks
```

### Create webhook

```
POST /api/v1/repos/{owner}/{name}/webhooks
```

**Request:**

```json
{
  "url": "https://ci.example.com/webhook",
  "events": ["push", "pull_request", "issue"],
  "secret": "my-webhook-secret"
}
```

### Update webhook

```
PATCH /api/v1/repos/{owner}/{name}/webhooks/{webhook_id}
```

### Delete webhook

```
DELETE /api/v1/repos/{owner}/{name}/webhooks/{webhook_id}
```

### Webhook deliveries

```
GET /api/v1/repos/{owner}/{name}/webhooks/{webhook_id}/deliveries
```

### Webhook payload

All webhook payloads include:

```json
{
  "event": "push",
  "repository": { "..." },
  "sender": { "..." },
  "delivery_id": "uuid",
  "timestamp": "2026-06-19T12:00:00Z"
}
```

The payload is signed with HMAC-SHA256 using the webhook secret. Verify via
the `X-CivitForge-Signature` header.

## Organizations

### List organizations

```
GET /api/v1/orgs
```

### Create organization

```
POST /api/v1/orgs
```

### Get organization

```
GET /api/v1/orgs/{org_name}
```

### Update organization

```
PATCH /api/v1/orgs/{org_name}
```

### Members

```
GET /api/v1/orgs/{org_name}/members
POST /api/v1/orgs/{org_name}/members
DELETE /api/v1/orgs/{org_name}/members/{username}
```

### Teams

```
GET /api/v1/orgs/{org_name}/teams
POST /api/v1/orgs/{org_name}/teams
```

## Users

### Get user

```
GET /api/v1/users/{username}
```

### Update profile

```
PATCH /api/v1/users/{username}
```

### SSH keys

```
GET /api/v1/users/{username}/keys
POST /api/v1/users/{username}/keys
DELETE /api/v1/users/{username}/keys/{key_id}
```

## Health

### Health check

```
GET /healthz
```

Returns `OK` when the server is healthy.

### Readiness check

```
GET /readyz
```

Returns `OK` when the server can accept traffic (database and Redis connected).

## Error responses

All error responses follow a consistent format:

```json
{
  "error": "not_found",
  "message": "Repository not found",
  "status": 404
}
```

### HTTP status codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (missing/invalid token) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 409 | Conflict (duplicate resource) |
| 422 | Unprocessable Entity (business logic error) |
| 429 | Too Many Requests (rate limit) |
| 500 | Internal Server Error |

## Pagination

List endpoints support pagination:

```
GET /api/v1/repos?page=2&per_page=50
```

Response includes:

```json
{
  "items": [...],
  "total": 150,
  "page": 2,
  "per_page": 50
}
```

## Rate limiting

When `RATE_LIMIT_MAX_REQUESTS` and `RATE_LIMIT_WINDOW_SECS` are configured,
rate-limited responses include:

```
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1750368060
```
