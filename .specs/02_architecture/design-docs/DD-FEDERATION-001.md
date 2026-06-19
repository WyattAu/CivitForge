# Multi-Region Federation Architecture

**Document ID:** DD-FEDERATION-001
**Status:** Proposed
**Target Version:** v3.0.0
**Author:** Autonomous Engineering

---

## 1. Problem Statement

CivitForge already implements ForgeFed ActivityPub federation for cross-instance
communication. However, true multi-region operation requires:

- **Active-active replication**: Users on different instances see changes within
  seconds, not minutes.
- **Conflict resolution**: Concurrent edits to issues/PRs across instances must
  converge without data loss.
- **Geo-distributed CI**: Runners in multiple regions serve nearby users.

## 2. Federation Protocol Extensions

### 2.1 ForgeFed + CRDT Layer

Standard ForgeFed ActivityPub uses "last writer wins" (LWW) semantics. We
extend it with CRDT (Conflict-free Replicated Data Type) metadata for
deterministic convergence:

```json
{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Update",
  "object": {
    "type": "Issue",
    "id": "https://forge-a.example/repo/1/issues/42",
    "title": "Bug in auth module",
    "status": "closed",
    "crdt": {
      "type": "LWWRegister",
      "timestamp": "2026-06-15T10:30:00Z",
      "node_id": "forge-a",
      "lamport_clock": 15234
    }
  }
}
```

### 2.2 Conflict Resolution Strategy

| Data Type | CRDT | Conflict Resolution |
|---|---|---|
| Issue title/body | LWW-Register | Highest (lamport_clock, node_id) wins |
| Issue labels | OR-Set (add-wins) | Union of add/remove operations |
| Issue assignees | OR-Set | Union of assignments |
| PR merge state | LWW-Register | Only the origin instance can merge |
| Comments | Grow-only Set | All comments retained, ordered by (clock, node) |
| Reactions | PN-Counter | Net positive = upvotes - downvotes |

### 2.3 Lamport Clock Synchronization

Each instance maintains a Lamport logical clock:

```rust
pub struct FederationClock {
    counter: AtomicU64,
    node_id: String,
}

impl FederationClock {
    pub fn tick(&self) -> LogicalTimestamp {
        let c = self.counter.fetch_add(1, Ordering::SeqCst);
        LogicalTimestamp {
            clock: c + 1,
            node_id: self.node_id.clone(),
        }
    }

    pub fn observe(&self, remote: &LogicalTimestamp) {
        loop {
            let local = self.counter.load(Ordering::SeqCst);
            let new = local.max(remote.clock) + 1;
            if self.counter.compare_exchange(local, new, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                break;
            }
        }
    }
}

impl Ord for LogicalTimestamp {
    // Total order: (clock, node_id) lexicographic comparison.
}
```

## 3. Outbox Delivery

### 3.1 Reliable Delivery Queue

Each instance maintains an outbox queue with at-least-once delivery:

```rust
pub struct FederationOutbox {
    db: PgPool,
    relay_workers: usize,
}

impl FederationOutbox {
    /// Enqueue an activity for delivery to all followers.
    pub async fn enqueue(&self, activity: Activity, followers: Vec<String>) -> Result<()> {
        for follower in followers {
            sqlx::query!(
                "INSERT INTO federation_outbox (activity_id, target, payload, status, attempts, next_retry)
                 VALUES ($1, $2, $3, 'pending', 0, NOW())",
                activity.id, follower, serde_json::to_value(&activity)?
            ).execute(&self.db).await?;
        }
        Ok(())
    }

    /// Background worker: deliver pending activities with exponential backoff.
    pub async fn relay_loop(&self) {
        loop {
            let pending = self.fetch_pending(100).await;
            for entry in pending {
                match self.deliver(&entry).await {
                    Ok(_) => self.mark_delivered(entry.id).await,
                    Err(e) => self.schedule_retry(entry.id, e).await,
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
```

### 3.2 HTTP Signature Verification

All federation traffic uses HTTP Signatures (draft-cavage-httpbis-signatures)
for authentication. The signature algorithm is negotiated via the actor's
public key endpoint:

```
GET /.well-known/webfinger?resource=acct:user@forge.example
→ links[rel=self].href = https://forge.example/users/user
→ GET https://forge.example/users/user
→ publicKey.publicKeyPem = "-----BEGIN PUBLIC KEY-----..."
```

## 4. Geo-Distributed CI Runners

### 4.1 Runner Registration

Runners register with a region tag:

```yaml
# runner config
CIVIT_RUNNER_TAGS: docker,linux,region-eu-west
CIVIT_RUNNER_REGION: eu-west-1
```

### 4.2 Pipeline Scheduling

Pipeline jobs specify region affinity:

```yaml
# .civitforge/pipeline.yml
jobs:
  test:
    runs-on: [docker, linux]
    region: nearest  # or specific: eu-west-1
    steps:
      - run: cargo test
```

The scheduler matches jobs to runners by region affinity:
1. If `region: nearest`, route to the runner with lowest latency to the repo's
   primary shard.
2. If `region: <specific>`, route only to runners in that region.
3. If no region specified, route to any available runner.

## 5. Consistency Guarantees

| Operation | Consistency | Rationale |
|---|---|---|
| Git push | Strong (origin authoritative) | Only the origin instance writes to the git repo |
| Issue/PR creation | Strong (origin authoritative) | Origin creates the ID |
| Issue/PR update | Eventual (CRDT) | Converges via LWW/OR-Set |
| Comments | Eventual (G-Set) | Always grows, never conflicts |
| Pipeline status | Strong (origin authoritative) | Only the running instance reports status |
| User profile | Strong (home instance) | Only home instance can edit |

## 6. Failure Modes

### 6.1 Network Partition

- Partitioned instances continue serving local users.
- Outbox queues accumulate undelivered activities.
- On heal, activities deliver in timestamp order.
- CRDT convergence resolves any conflicts.

### 6.2 Instance Recovery

- On restart, drain outbox queue before accepting new writes.
- Verify inbox deduplication (idempotent apply by activity_id).
- Re-sync any missed activities via `Undo` + replay.

## 7. Metrics

| Metric | Target |
|---|---|
| Cross-instance propagation delay | < 5 seconds |
| Conflict rate | < 0.1% of activities |
| Outbox queue depth | < 100 messages |
| Delivery success rate | > 99.9% |
