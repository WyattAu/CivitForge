# CivitForge API Reference

REST API for CivitForge v0.8.0-alpha. Base URL: `http://localhost:8080`

## Authentication

Most API endpoints require a JWT token in the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

Obtain a token via `POST /api/v1/auth/login`.

## Response Format

All responses are JSON. Error responses follow:

```json
{
  "error": "description of the error"
}
```

## Endpoints

### Health & Readiness

#### `GET /healthz`
Liveness probe. Returns `OK` (200) if the server process is running.

**Response:** `OK` (plain text)

---

#### `GET /ready`
Readiness probe. Returns `OK` (200) if the server is ready for traffic.

**Response:** `OK` (plain text)

---

#### `GET /api/v1/health`
API health check. Returns `OK` (200).

**Response:** `OK` (plain text)

---

### Authentication

#### `POST /api/v1/auth/login`
Authenticate and receive a JWT token.

**Request body:**
```json
{
  "username": "alice",
  "password": "password123"
}
```

**Response (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2026-06-02T00:00:00Z"
}
```

---

#### `GET /api/v1/auth/me`
Get the current authenticated user's profile. Requires `Authorization: Bearer <token>`.

**Response (200):**
```json
{
  "id": "u1",
  "username": "alice",
  "email": "alice@example.com",
  "role": "admin",
  "created_at": "2026-06-01T00:00:00Z"
}
```

---

### Users

#### `GET /api/v1/users`
List all users.

**Query parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 100 | Maximum results |

**Response (200):**
```json
[
  {
    "id": "u1",
    "username": "alice",
    "email": "alice@example.com",
    "created_at": "2026-06-01T00:00:00Z"
  }
]
```

---

#### `POST /api/v1/users`
Create a new user.

**Request body:**
```json
{
  "username": "bob",
  "email": "bob@example.com",
  "password": "secure-password"
}
```

**Response (201):** Created user object.

---

#### `GET /api/v1/users/{id}`
Get a single user by ID.

**Response (200):** User object.

---

#### `PATCH /api/v1/users/{id}`
Update a user.

**Request body (partial):**
```json
{
  "email": "new-email@example.com"
}
```

**Response (200):** Updated user object.

---

#### `DELETE /api/v1/users/{id}`
Delete a user.

**Response (204):** No content on success.

---

### Organizations

#### `GET /api/v1/orgs`
List all organizations.

**Response (200):** Array of organization objects.

---

#### `POST /api/v1/orgs`
Create an organization.

**Request body:**
```json
{
  "name": "my-org",
  "description": "My organization"
}
```

**Response (201):** Created organization object.

---

#### `GET /api/v1/orgs/{id}`
Get a single organization.

**Response (200):** Organization object.

---

#### `PATCH /api/v1/orgs/{id}`
Update an organization.

**Response (200):** Updated organization object.

---

### Repositories

#### `GET /api/v1/repos`
List all repositories.

**Query parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 100 | Maximum results |

**Response (200):**
```json
[
  {
    "id": "r1",
    "owner": "alice",
    "name": "my-project",
    "description": "A project",
    "visibility": "public",
    "created_at": "2026-06-01T00:00:00Z"
  }
]
```

---

#### `POST /api/v1/repos`
Create a new repository.

**Request body:**
```json
{
  "name": "new-project",
  "description": "Project description",
  "visibility": "public"
}
```

**Response (201):** Created repository object.

---

#### `GET /api/v1/repos/{owner}/{name}`
Get a single repository.

**Response (200):** Repository object.

---

#### `DELETE /api/v1/repos/{owner}/{name}`
Delete a repository and its storage.

**Response (204):** No content on success.

---

#### `GET /api/v1/repos/{owner}/{name}/commits`
List commits for a repository.

**Response (200):**
```json
[
  {
    "id": "abc1234",
    "message": "Initial commit",
    "author": "alice",
    "timestamp": "2026-06-01T00:00:00Z"
  }
]
```

---

### SSH Keys

#### `GET /api/v1/users/{user_id}/ssh-keys`
List SSH keys for a user.

**Response (200):** Array of SSH key objects.

---

#### `POST /api/v1/users/{user_id}/ssh-keys`
Add an SSH key.

**Request body:**
```json
{
  "title": "my-laptop",
  "public_key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI..."
}
```

**Response (201):** Created SSH key object.

---

#### `DELETE /api/v1/ssh-keys/{key_id}`
Delete an SSH key.

**Response (204):** No content on success.

---

### WebSocket

#### `GET /api/v1/ws`
Upgrade to WebSocket for real-time event streaming.

**Protocol:** Standard WebSocket (RFC 6455). Messages are JSON-encoded events.

**Event types:**
- `repo.created`
- `repo.deleted`
- `pipeline.started`
- `pipeline.completed`
- `notification.dispatched`

---

### Git Smart HTTP

#### `GET /{owner}/{name}/info/refs`
Git reference advertisement (smart HTTP protocol).

**Response:** Git protocol text (Content-Type: `application/x-git-upload-pack-advertisement`)

---

#### `POST /{owner}/{name}/git-upload-pack`
Git pack upload (clone/fetch).

**Request body:** Git protocol handshake.

---

#### `POST /{owner}/{name}/git-receive-pack`
Git pack receive (push).

**Request body:** Git protocol handshake with pack data.

---

## Rate Limits

| Limit | Value |
|-------|-------|
| Max concurrent connections | No hard limit (configurable via reverse proxy) |
| SSH auth rate limit | 5 attempts per second per IP |
| JWT token expiry | Configurable via `JWT_EXPIRY_HOURS` (default: 24h) |

## CORS

CORS is enabled by default with `allow-origin: *`, `allow-methods: *`, `allow-headers: *`. Restrict via reverse proxy in production.

## Error Codes

| HTTP Status | Meaning |
|-------------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No content |
| 400 | Bad request (invalid input) |
| 401 | Unauthorized (missing/invalid JWT) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not found |
| 409 | Conflict (duplicate resource) |
| 422 | Validation error |
| 500 | Internal server error |
