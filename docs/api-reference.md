# API Reference

## Authentication

All API endpoints (except login/register) require authentication via one of:

### JWT Bearer Token

```
Authorization: Bearer <jwt-token>
```

Tokens are obtained from `/api/v1/auth/login` and expire after 24 hours (configurable).

### Personal Access Token

```
Authorization: Token <access-token>
```

Tokens are created via `/api/v1/user/tokens` and do not expire unless revoked.

---

## REST API v1

Base URL: `/api/v1`

### Authentication

#### POST /auth/register

Create a new user account.

Request:
```json
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "secure-password"
}
```

Response (201):
```json
{
  "id": "uuid",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-01-01T00:00:00Z"
}
```

#### POST /auth/login

Authenticate and receive a JWT.

Request:
```json
{
  "username": "alice",
  "password": "secure-password"
}
```

Response (200):
```json
{
  "token": "eyJ...",
  "expires_at": "2025-01-02T00:00:00Z"
}
```

#### POST /auth/logout

Invalidate the current session.

Response (204): No content.

#### POST /auth/refresh

Refresh the current JWT token.

Request:
```json
{
  "refresh_token": "eyJ..."
}
```

Response (200):
```json
{
  "token": "eyJ...",
  "expires_at": "2025-01-02T00:00:00Z"
}
```

### Users

#### GET /user

Get the authenticated user's profile.

Response (200):
```json
{
  "id": "uuid",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-01-01T00:00:00Z",
  "public_keys": []
}
```

#### GET /users/:username

Get a user's public profile.

Response (200):
```json
{
  "id": "uuid",
  "username": "alice",
  "created_at": "2025-01-01T00:00:00Z"
}
```

#### POST /user/tokens

Create a personal access token.

Request:
```json
{
  "name": "ci-token",
  "scopes": ["repo:read", "pipeline:write"]
}
```

Response (201):
```json
{
  "id": "uuid",
  "name": "ci-token",
  "token": "cpt_...",
  "scopes": ["repo:read", "pipeline:write"],
  "created_at": "2025-01-01T00:00:00Z"
}
```

### Repositories

#### GET /repos

List repositories accessible to the authenticated user.

Query parameters:
- `page` (default: 1)
- `per_page` (default: 30, max: 100)
- `sort` (default: `updated`, options: `updated`, `created`, `name`)

Response (200):
```json
{
  "items": [
    {
      "id": "uuid",
      "name": "my-project",
      "owner": { "username": "alice" },
      "description": "A project",
      "visibility": "public",
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "per_page": 30
}
```

#### POST /repos

Create a new repository.

Request:
```json
{
  "name": "my-project",
  "description": "A project",
  "visibility": "public"
}
```

Response (201):
```json
{
  "id": "uuid",
  "name": "my-project",
  "owner": { "username": "alice" },
  "description": "A project",
  "visibility": "public",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

#### GET /repos/:owner/:name

Get a repository.

Response (200): Same as repository object above.

#### PATCH /repos/:owner/:name

Update repository settings.

Request:
```json
{
  "description": "Updated description",
  "visibility": "private"
}
```

Response (200): Updated repository object.

#### DELETE /repos/:owner/:name

Delete a repository.

Response (204): No content.

### Branches and Tags

#### GET /repos/:owner/:name/branches

List branches.

Response (200):
```json
{
  "items": [
    {
      "name": "main",
      "commit": { "id": "sha256", "message": "Initial commit" },
      "default": true
    }
  ]
}
```

#### GET /repos/:owner/:name/tags

List tags.

Response (200):
```json
{
  "items": [
    {
      "name": "v1.0.0",
      "commit": { "id": "sha256", "message": "Release 1.0.0" }
    }
  ]
}
```

### Pipelines

#### GET /repos/:owner/:name/pipelines

List CI/CD pipelines for a repository.

Query parameters:
- `page`, `per_page`
- `status` (filter: `pending`, `running`, `success`, `failed`)

Response (200):
```json
{
  "items": [
    {
      "id": "uuid",
      "status": "success",
      "trigger": "push",
      "ref": "refs/heads/main",
      "commit": { "id": "sha256", "message": "Add tests" },
      "created_at": "2025-01-01T00:00:00Z",
      "duration_seconds": 45
    }
  ],
  "total": 10
}
```

#### POST /repos/:owner/:name/pipelines

Trigger a pipeline run.

Request:
```json
{
  "ref": "refs/heads/main"
}
```

Response (201): Pipeline object.

#### GET /repos/:owner/:name/pipelines/:id

Get pipeline details with job status.

Response (200):
```json
{
  "id": "uuid",
  "status": "running",
  "jobs": [
    {
      "id": "uuid",
      "name": "test",
      "status": "running",
      "started_at": "2025-01-01T00:00:05Z"
    }
  ]
}
```

### Activity

#### GET /repos/:owner/:name/activity

Get repository activity feed.

Response (200):
```json
{
  "items": [
    {
      "type": "push",
      "actor": { "username": "alice" },
      "description": "pushed 3 commits to main",
      "created_at": "2025-01-01T00:00:00Z"
    }
  ]
}
```

---

## WebSocket API

Endpoint: `ws://host/ws`

### Connection

Connect with JWT token as query parameter:
```
ws://host/ws?token=<jwt-token>
```

### Subscription Format

Subscribe to events:
```json
{
  "action": "subscribe",
  "channel": "repo:uuid"
}
```

Unsubscribe:
```json
{
  "action": "unsubscribe",
  "channel": "repo:uuid"
}
```

### Event Types

| Event | Payload | Description |
|-------|---------|-------------|
| `push` | `{ repo_id, actor, commits[], ref }` | New commits pushed |
| `pipeline.start` | `{ repo_id, pipeline_id }` | Pipeline started |
| `pipeline.complete` | `{ repo_id, pipeline_id, status }` | Pipeline finished |
| `comment` | `{ repo_id, author, body }` | New comment |
| `issue` | `{ repo_id, action, issue }` | Issue created/updated |

### Heartbeat

Server sends `{"type": "ping"}` every 30 seconds. Client must respond with `{"type": "pong"}` within 10 seconds.

---

## gRPC API

### Services

#### RepositoryService

```protobuf
service RepositoryService {
  rpc GetRepository(GetRepositoryRequest) returns (Repository);
  rpc ListBranches(ListBranchesRequest) returns (stream Branch);
  rpc GetTree(GetTreeRequest) returns (stream TreeEntry);
}
```

#### PipelineService

```protobuf
service PipelineService {
  rpc GetPipeline(GetPipelineRequest) returns (Pipeline);
  rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
  rpc TriggerPipeline(TriggerPipelineRequest) returns (Pipeline);
}
```

---

## Error Responses

All errors return a JSON body:

```json
{
  "error": "human-readable error message",
  "code": "ERROR_CODE"
}
```

### Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `BAD_REQUEST` | Malformed request body or parameters |
| 401 | `UNAUTHORIZED` | Missing or invalid authentication |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 404 | `NOT_FOUND` | Resource does not exist |
| 409 | `CONFLICT` | Resource already exists |
| 422 | `VALIDATION_ERROR` | Input validation failed |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Unexpected server error |
| 503 | `SERVICE_UNAVAILABLE` | Dependent service down |
