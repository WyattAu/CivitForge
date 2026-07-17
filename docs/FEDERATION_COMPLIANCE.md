# Federation Protocol Compliance

## Protocol Compliance Matrix

| Feature | Status | Spec | Notes |
|---|---|---|---|
| ActivityPub Object Model | Implemented | ActivityPub | Create/Update/Delete activities |
| ActivityPub Activity Types | Implemented | ActivityPub | Create, Update, Delete, Follow, Undo, Accept, Reject, Add, Like, Announce |
| HTTP Signatures | Implemented | HTTP Signatures (draft-cavage) | Ed25519, RSA-SHA256, ECDSA-P256, HMAC-SHA256 |
| WebFinger Discovery | Implemented | RFC 7033 | acct: URI scheme, fallback on unreachable |
| Inbox Processing | Implemented | ActivityPub | Idempotent receive, retry with backoff, max retries |
| Outbox Processing | Implemented | ActivityPub | Enqueue, in-flight, delivered, permanent failure |
| Actor Profiles | Implemented | ActivityPub | Person, Organization, Application, Service, Group |
| LD+JSON Context | Implemented | ActivityPub | `application/activity+json` content type |
| ForgeFed Extensions | Implemented | ForgeFed | Repository, Fork, Star, Issue, PR, Review, Comment |
| Cross-Instance Identity | Implemented | ForgeFed | WebFinger + cache resolution |
| Idempotency | Implemented | ActivityPub | Duplicate detection via activity ID |

## Test Coverage

### Unit Tests (in-module)

- `activitypub.rs`: 7 tests — actor validation, activity processing, WebFinger response
- `http_signatures.rs`: 19 tests — algorithm display/parse, sign/verify (Ed25519, HMAC, RSA, ECDSA), LD signatures
- `webfinger.rs`: 8 tests — resolve, parse JSON, HTTP signature legacy/Ed25519, create/verify roundtrip
- `inbox_outbox.rs`: 16 tests — receive, idempotency, process, retry, outbox enqueue/deliver/fail
- `forgefed.rs`: 25 tests — all activity types, idempotency, outbox mapping, identity resolver

### Integration Tests (protocol_compliance.rs)

- **ActivityPub Serialization** (8 tests): Create/Update/Delete roundtrip, object variants, type names, fixture parsing
- **HTTP Signatures** (7 tests): Ed25519 roundtrip, HMAC roundtrip, tamper rejection, wrong key, header value roundtrip, expiry, algorithm parse
- **WebFinger Discovery** (5 tests): Fallback response, empty domain/username rejection, structure validation, fixture, handler response
- **Inbox/Outbox Handling** (6 tests): Receive & process, idempotency, retry on failure, outbox enqueue/deliver, backoff, permanent failure
- **Actor Profile Resolution** (5 tests): Valid actor, empty id/inbox/username rejection, fixture, serialization, type variants
- **Collection Pages** (3 tests): Identity resolver, cache operations, federation URI format
- **LD+JSON Context** (4 tests): Create context, actor context, object types, self link type
- **Error Handling** (7 tests): Empty actor, no recipient, missing repo id, missing reviewer, result variants, all activity types
- **ForgeFed Activity Processing** (6 tests): Create repo, fork, issue, PR, idempotency, outbox mapping

**Total: 100 tests** (58 unit + 42 integration)

## Known Limitations

1. **No streaming/inbox forwarding** — Incoming activities are queued but not pushed to followers via Signed Fetch
2. **No collection page pagination** — Followers/following/outbox endpoints return full collections without `first`/`next` links
3. **No LD-Signature verification** — LD-Signature support is partial; HTTP Signatures (draft-cavage) are fully supported
4. **No shared inbox fanout** — Activities are delivered to each follower's inbox individually
5. **No transient error recovery with jitter** — Backoff uses deterministic jitter, not cryptographic randomness
6. **No ActivityPub addressing resolution** — `to`/`cc` fields are not resolved to actor inboxes

## Interoperability Notes

### Tested With

| Software | Status | Notes |
|---|---|---|
| Mastodon | Partial | WebFinger and actor resolution work; ForgeFed extensions not recognized |
| Gitea | Partial | WebFinger works; ForgeFed activity types may not be recognized |
| Forgejo | Partial | WebFinger works; ForgeFed extensions require Forgejo's federation support |
| Pleroma/Akkoma | Untested | Should work for basic ActivityPub activities |
| Misskey | Untested | Should work for basic ActivityPub activities |

### ActivityPub Compliance

- **Content-Type**: `application/activity+json` and `application/ld+json; profile="https://www.w3.org/ns/activitystreams"`
- **Required fields**: `@context`, `type`, `id`, `actor`, `object`, `published`, `to`
- **Signature algorithms**: Ed25519 (preferred), RSA-SHA256, ECDSA-P256, HMAC-SHA256

### ForgeFed Compliance

- **Activity types**: CreateRepository, ForkRepository, StarRepository, FollowUser, CreateIssue, CreatePullRequest, ReviewPullRequest, Comment, Like, Accept, Reject, Undo
- **Object types**: Repository, Issue, PullRequest, Note
- **Actor types**: Person, Organization, Application, Service, Group
