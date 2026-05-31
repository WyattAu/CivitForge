# ADR-0006: ActivityPub Federation

## Status

Accepted

## Context

CivitForge is a federated forge, meaning instances can interact across organizational boundaries. Federation is a core requirement.

## Decision

Implement federation using the ActivityPub protocol (W3C Recommendation).

## Considerations

- ActivityPub is the W3C standard for decentralized social/federated applications
- Compatible with existing Fediverse ecosystem (Mastodon, Lemmy, Forgejo)
- Provides inbox/outbox delivery model for federated actions
- JSON-LD based content addressing
- WebFinger for account discovery
- HTTP Signatures for authentication between instances

## Implementation

- `civit-brain` handles federation protocol logic
- Actor objects map to users/organizations
- Repository activities (push, fork, star) map to ActivityPub activities
- Signature verification via `civit-crypto`

## Consequences

- Interoperable with other ActivityPub forges
- Federation state requires eventual consistency model
- HTTP Signature verification adds latency to federated requests
- Inbox polling or WebSub for push delivery
