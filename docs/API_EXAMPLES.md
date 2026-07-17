# CivitForge API Examples

Base URL: `http://localhost:9091`

---

## Authentication

### Login (auto-registers on first use)

```bash
curl -X POST http://localhost:9091/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","display_name":"Alice"}'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

Save the token for subsequent requests:

```bash
export TOKEN="<token from response>"
```

---

## Repository Operations

### List Repositories

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos
```

### Create a Repository

```bash
curl -X POST http://localhost:9091/api/v1/repos \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "my-project",
    "owner": "alice",
    "description": "A sample repository",
    "visibility": "public"
  }'
```

### Get a Repository

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project
```

### Delete a Repository

```bash
curl -X DELETE \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project
```

---

## Issue Management

### List Issues

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project/issues
```

### Create an Issue

```bash
curl -X POST \
  http://localhost:9091/api/v1/repos/alice/my-project/issues \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Fix login bug",
    "body": "Users cannot log in with SSO when MFA is enabled."
  }'
```

### Get a Single Issue

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project/issues/1
```

---

## Pipeline Triggers

### List Pipelines

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project/pipelines
```

### Trigger a Pipeline

```bash
curl -X POST \
  http://localhost:9091/api/v1/repos/alice/my-project/pipelines \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "branch": "main",
    "commit_sha": "abc1234",
    "trigger": "push"
  }'
```

---

## Webhook Setup

### Register a Webhook

```bash
curl -X POST \
  http://localhost:9091/api/v1/repos/alice/my-project/hooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "url": "https://example.com/webhook",
    "events": ["push", "pull_request", "issues"],
    "active": true,
    "secret": "my-webhook-secret"
  }'
```

### List Webhooks

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project/hooks
```

### Delete a Webhook

```bash
curl -X DELETE \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:9091/api/v1/repos/alice/my-project/hooks/1
```

---

## Health & Readiness

```bash
curl http://localhost:9091/healthz
curl http://localhost:9091/ready
curl http://localhost:9091/api/v1/health
```
