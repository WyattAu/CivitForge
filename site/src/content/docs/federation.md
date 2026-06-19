---
title: Federation Protocol
description: ActivityPub, ForgeFed, multi-master replication, and vector clocks in CivitForge.
---

## Overview

CivitForge implements the ForgeFed protocol, an extension of ActivityPub
designed for software forge federation. This enables cross-instance
collaboration: issues, pull requests, code search, and notifications can
span multiple CivitForge instances.

Federation is opt-in and disabled by default. When enabled, each instance
operates as an independent ActivityPub server with a unique identity.

## Protocol Stack

```
┌─────────────────────────────────────────┐
│            ForgeFed Protocol            │
├─────────────────────────────────────────┤
│         ActivityPub (W3C)               │
├─────────────────────────────────────────┤
│       HTTP Signatures (RFC 9421)        │
├─────────────────────────────────────────┤
│            WebFinger (RFC 7033)          │
└─────────────────────────────────────────┘
```

### ActivityPub

ActivityPub is the base protocol for decentralized social networking.
CivitForge implements:

- **Server-to-Server (S2S)**: Inter-instance communication
- **Client-to-Server (C2S)**: Optional, for external clients
- **Actors**: Users, repositories, organizations
- **Activities**: Create, Update, Delete, Follow, Announce, Undo
- **Inbox/Outbox**: Ordered collection endpoints

### ForgeFed

ForgeFed extends ActivityPub with forge-specific object types:

- **Repository**: Represents a Git repository
- **Issue**: Federated issue tracking
- **PullRequest**: Cross-instance code review
- **Comment**: Discussion threads on issues/PRs
- **Branch**: Branch references

### HTTP Signatures

All inter-instance requests are signed using HTTP Signatures (RFC 9421):

- Algorithm: `rsa-sha256` or `ed25519`
- Key ID: `<instance_url>/keys/<key_id>`
- Signed headers: `(request-target)`, `host`, `date`, `digest`
- Verification: Each instance maintains a public key registry

### WebFinger

Instance discovery uses WebFinger (RFC 7033):

```
GET /.well-known/webfinger?resource=acct:user@forge.example.com
```

Response:

```json
{
  "subject": "acct:user@forge.example.com",
  "links": [
    {
      "rel": "self",
      "type": "application/activity+json",
      "href": "https://forge.example.com/users/user"
    }
  ]
}
```

## Configuration

### Environment variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `FEDERATION_ENABLED` | bool | `false` | Enable federation |
| `FEDERATION_INSTANCE_ID` | string | -- | Unique instance identifier |
| `FEDERATION_INSTANCE_DOMAIN` | string | -- | Public domain name |

Both `FEDERATION_INSTANCE_ID` and `FEDERATION_INSTANCE_DOMAIN` are required
when federation is enabled.

### Instance identity

Each instance generates an RSA keypair at first startup:

- Private key: `<storage_path>/federation/private_key.pem`
- Public key: `<storage_path>/federation/public_key.pem`

The public key is published at:

```
GET /federation/keys/<instance_id>
```

## Federation Endpoints

### Outbox

```
POST /federation/outbox
```

Sends activities to remote instances. The outbox is a persistent queue backed
by Redis. Failed deliveries are retried with exponential backoff.

### Inbox

```
POST /federation/inbox
```

Receives activities from remote instances. Activities are validated:
1. HTTP signature verification
2. Actor resolution via WebFinger
3. Activity schema validation
4. Idempotency check (dedup by activity ID)

### Actor endpoints

```
GET /federation/actors/<actor_id>
```

Returns the ActivityPub actor representation:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "id": "https://forge.example.com/users/alice",
  "preferredUsername": "alice",
  "inbox": "https://forge.example.com/federation/inbox",
  "outbox": "https://forge.example.com/federation/outbox",
  "publicKey": {
    "id": "https://forge.example.com/keys/instance-1",
    "owner": "https://forge.example.com/users/alice",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----\n..."
  }
}
```

## Multi-Master Replication

CivitForge supports multi-master replication for high-availability
federation. Each instance maintains its own database and syncs state
via federation activities.

### Sync engine

The `IncrementalSyncEngine` in `civit-runner/src/sync.rs` handles
replication:

```
┌──────────────┐     ┌──────────────┐
│  Instance A  │────>│  Instance B  │
│              │<────│              │
└──────┬───────┘     └──────┬───────┘
       │                    │
       ▼                    ▼
  ┌─────────┐          ┌─────────┐
  │  DB A   │          │  DB B   │
  └─────────┘          └─────────┘
```

### Sync protocol

1. Instance A creates an activity (e.g., new issue)
2. Activity is signed and sent to Instance B's inbox
3. Instance B verifies the signature
4. Activity is applied to Instance B's database
5. Instance B sends an `Accept` or `Reject` back

### Conflict resolution

Conflicts are resolved using vector clocks:

```json
{
  "vector_clock": {
    "instance-a": 42,
    "instance-b": 37
  }
}
```

Resolution rules:
1. If vector clocks are equal, last-writer-wins (LWW) by timestamp
2. If one clock dominates, that version wins
3. If clocks are concurrent, the activity with the higher instance ID wins
4. Conflicts are logged but do not block delivery

### Checkpointing

The sync engine checkpoints progress to prevent re-processing:

```
federation:sync:<instance_id>:checkpoint = <last_activity_id>
```

On restart, the engine resumes from the last checkpoint.

## Activity Types

### Repository activities

| Activity | Object | Description |
|----------|--------|-------------|
| `Create` | Repository | New repository created |
| `Update` | Repository | Repository metadata changed |
| `Delete` | Repository | Repository deleted |

### Issue activities

| Activity | Object | Description |
|----------|--------|-------------|
| `Create` | Issue | New issue opened |
| `Update` | Issue | Issue edited |
| `Add` | Comment | Comment added |
| `Remove` | Issue | Issue closed |

### Pull request activities

| Activity | Object | Description |
|----------|--------|-------------|
| `Create` | PullRequest | New PR opened |
| `Update` | PullRequest | PR edited |
| `Offer` | MergeRequest | PR merge proposed |
| `Accept` | PullRequest | PR merged |
| `Reject` | PullRequest | PR closed without merge |

### Comment activities

| Activity | Object | Description |
|----------|--------|-------------|
| `Create` | Comment | New comment |
| `Update` | Comment | Comment edited |
| `Delete` | Comment | Comment deleted |

## WebFinger Discovery

To discover a remote user:

```
GET /.well-known/webfinger?resource=acct:alice@forge.example.com
```

To discover a remote repository:

```
GET /.well-known/webfinger?resource=urn:forgeforge:repo:alice/my-project
```

## Trust model

- Each instance has a unique RSA keypair
- Keys are published at `/federation/keys/<id>`
- Activities must be signed by the actor's key
- Unknown instances are discovered via WebFinger
- There is no central authority; trust is peer-to-peer

## Rate limiting

Federation endpoints have separate rate limits:

| Endpoint | Limit | Window |
|----------|-------|--------|
| Inbox | 100 requests/minute/instance | 60s |
| Outbox | 50 requests/minute | 60s |
| WebFinger | 20 requests/minute/IP | 60s |

## Monitoring

Federation health can be monitored via:

- `GET /healthz` -- includes federation status
- `GET /federation/stats` -- delivery queue depth, sync lag
- Prometheus metrics (when `serviceMonitor` is enabled in Helm)

Key metrics:
- `civit_federation_delivery_total` -- total deliveries
- `civit_federation_delivery_failed_total` -- failed deliveries
- `civit_federation_sync_lag_seconds` -- replication lag
- `civit_federation_inbox_received_total` -- received activities
