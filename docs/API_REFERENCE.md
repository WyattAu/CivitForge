# CivitForge API Reference

REST API for CivitForge v2.2.0. Base URL: `http://localhost:9091/api/v1`

## Authentication

All endpoints (except register/login) require a JWT bearer token:

```
Authorization: Bearer <jwt-token>
```

Tokens are obtained via `POST /api/v1/auth/login`. Auth is register-on-login: registering a new user simultaneously creates the account and returns a JWT.

## Response Format

All responses are JSON. Error responses:

```json
{
  "error": "description of the error"
}
```

## Endpoints

### Health and readiness

| Method | Path | Description |
|--------|------|-------------|
| GET | `/healthz` | Liveness probe. Returns `OK` (200). |
| GET | `/ready` | Readiness probe. Returns `OK` (200). |
| GET | `/api/v1/health` | API health check. Returns `OK` (200). |

### Authentication

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/auth/register` | Register new user (username, email, password). Returns user object (201). |
| POST | `/api/v1/auth/login` | Authenticate. Returns `{ "token": "eyJ...", "expires_at": "..." }` (200). |
| POST | `/api/v1/auth/logout` | Invalidate current session (204). |
| POST | `/api/v1/auth/refresh` | Refresh JWT token. Requires `refresh_token` in body (200). |
| GET | `/api/v1/auth/me` | Get current authenticated user profile. Requires Bearer token (200). |

### Users

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/users` | List users. Query: `limit` (default 100). |
| POST | `/api/v1/users` | Create user (username, email, password) (201). |
| GET | `/api/v1/users/{id}` | Get user by ID (200). |
| PATCH | `/api/v1/users/{id}` | Update user (partial) (200). |
| DELETE | `/api/v1/users/{id}` | Delete user (204). |
| GET | `/api/v1/user` | Get own profile (200). |
| POST | `/api/v1/user/tokens` | Create personal access token (201). |

### Organizations

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/orgs` | List organizations (200). |
| POST | `/api/v1/orgs` | Create organization (name, description) (201). |
| GET | `/api/v1/orgs/{id}` | Get organization (200). |
| PATCH | `/api/v1/orgs/{id}` | Update organization (200). |

### Repositories

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/repos` | List repos. Query: `limit`, `sort` (updated, created, name), `page`, `per_page`. |
| POST | `/api/v1/repos` | Create repository (name, description, visibility) (201). |
| GET | `/api/v1/repos/{owner}/{name}` | Get repository (200). |
| PATCH | `/api/v1/repos/{owner}/{name}` | Update repository settings (200). |
| DELETE | `/api/v1/repos/{owner}/{name}` | Delete repository (204). |
| GET | `/api/v1/repos/{owner}/{name}/commits` | List commits (200). |
| GET | `/api/v1/repos/{owner}/{name}/branches` | List branches (200). |
| GET | `/api/v1/repos/{owner}/{name}/tags` | List tags (200). |
| GET | `/api/v1/repos/{owner}/{name}/activity` | Activity feed (200). |

### Pipelines

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/repos/{owner}/{name}/pipelines` | List pipelines. Query: `status` (pending, running, success, failed), `page`, `per_page`. |
| POST | `/api/v1/repos/{owner}/{name}/pipelines` | Trigger pipeline run (201). |
| GET | `/api/v1/repos/{owner}/{name}/pipelines/{id}` | Get pipeline with job status (200). |

### Issues

18 endpoints for CRUD, state machine (open, in_progress, closed, reopen), timeline, comments, labels, milestones, assignees.

### Wiki

9 endpoints for page CRUD, page history with diff, raw content export, full-text search.

### Code search

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/search` | Global full-text search across repos. Query: `q`, `language`. |
| GET | `/api/v1/repos/{owner}/{name}/search` | Per-repo search. |

### OCI container registry

20 OCI Distribution Spec v1.1 endpoints under `/v2/`:
- `/v2/_catalog`, `/v2/{name}/tags/list`
- `/v2/{name}/blobs/{digest}` (HEAD, GET, PUT, DELETE)
- `/v2/{name}/manifests/{reference}` (HEAD, GET, PUT, DELETE)

8 management endpoints:
- List images, tags, layers, SBOM, vulnerability scans, RBAC policies, trigger garbage collection

### Runners

11 runner management endpoints. Runner protocol (internal API):
- `POST /api/internal/runners/register` -- register runner
- `GET /api/internal/runners/tasks` -- poll for jobs
- `POST /api/internal/runners/tasks/{id}/claim` -- claim job
- `POST /api/internal/runners/tasks/{id}/logs` -- upload log chunks
- `POST /api/internal/runners/tasks/{id}/status` -- update status
- `POST /api/internal/runners/tasks/{id}/complete` -- mark complete

### SSH keys

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/users/{user_id}/ssh-keys` | List SSH keys (200). |
| POST | `/api/v1/users/{user_id}/ssh-keys` | Add SSH key (title, public_key) (201). |
| DELETE | `/api/v1/ssh-keys/{key_id}` | Delete SSH key (204). |

### WebSocket

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/ws` | WebSocket event stream. Connect with `?token=<jwt>`. |

Event types: `repo.created`, `repo.deleted`, `pipeline.started`, `pipeline.completed`, `notification.dispatched`, `push`, `comment`, `issue`.

Heartbeat: server sends `{"type": "ping"}` every 30s, client must respond with `{"type": "pong"}` within 10s.

### Git smart HTTP

| Method | Path | Description |
|--------|------|-------------|
| GET | `/{owner}/{name}/info/refs` | Git reference advertisement (smart HTTP). |
| POST | `/{owner}/{name}/git-upload-pack` | Git pack upload (clone/fetch). |
| POST | `/{owner}/{name}/git-receive-pack` | Git pack receive (push). |

## Rate Limits

| Limit | Value |
|-------|-------|
| SSH auth rate limit | 5 attempts/second/IP |
| JWT token expiry | Configurable via `JWT_EXPIRY_HOURS` (default 24h) |

## Error Codes

| HTTP Status | Description |
|-------------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No content |
| 400 | Bad request (invalid input) |
| 401 | Unauthorized (missing/invalid JWT) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not found |
| 409 | Conflict (duplicate resource) |
| 422 | Validation error |
| 429 | Rate limited |
| 500 | Internal server error |
