- `activitypub.rs`: 7 tests; actor validation, activity processing, WebFinger response
- `http_signatures.rs`: 19 tests; algorithm display/parse, sign/verify (Ed25519, HMAC, RSA, ECDSA), LD signatures
- `webfinger.rs`: 8 tests; resolve, parse JSON, HTTP signature legacy/Ed25519, create/verify roundtrip
- `inbox_outbox.rs`: 16 tests; receive, idempotency, process, retry, outbox enqueue/deliver/fail
- `forgefed.rs`: 25 tests; all activity types, idempotency, outbox mapping, identity resolver
1. **No streaming/inbox forwarding**; Incoming activities are queued but not pushed to followers via Signed Fetch
2. **No collection page pagination**; Followers/following/outbox endpoints return full collections without `first`/`next` links
3. **No LD-Signature verification**; LD-Signature support is partial; HTTP Signatures (draft-cavage) are fully supported
4. **No shared inbox fanout**; Activities are delivered to each follower's inbox individually
5. **No transient error recovery with jitter**; Backoff uses deterministic jitter, not cryptographic randomness
6. **No ActivityPub addressing resolution**; `to`/`cc` fields are not resolved to actor inboxes