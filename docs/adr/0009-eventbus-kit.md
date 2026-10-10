# ADR-0009: eventbus-kit — bounded delivery for the event bus

- Status: accepted (2026-10); kit work landed as `eventbus-kit` 0.4.0
- Research inputs: Lagom / Designing Reactive Systems (Lightbend),
  Reactive Manifesto, Bonér's *Microservices Design*

## Context

An audit of what CivitForge still hand-rolls found `crates/civit-core/src/events`
— 2,514 lines across bus, websocket, log_stream, websocket_scaler — as the
largest hand-rolled subsystem, with 52 subscribe sites and 7 publish sites.

Reading `EventBus::publish` against the actor/reactive literature shows the
gap is not size, it is semantics. It is synchronous: it clones the event into
a mutex-guarded ring, then calls every subscriber **inline** while holding a
DashMap read guard. So:

- One slow subscriber blocks the publisher, and the publisher is a request
  handler — a client-visible latency cliff.
- A panicking subscriber takes down the publish path.
- There is no backpressure policy: no bounded per-subscriber queue, no
  overflow decision, nowhere to observe saturation.
- `replay` scans the shared ring under the same lock publishes contend on.

The reactive literature's answer is consistent: message-driven design with no
shared mutable state, backpressure as a first-class concern, and failure
isolation designed rather than hoped for.

## Correction to the first draft

The first draft of this ADR proposed extracting a new `eventbus-kit` from
scratch. That was wrong: **`eventbus-kit` already exists** (`WyattAu/eventbus`,
published, 0.3.5 at the time of writing). The audit should have checked the
owner's existing catalogue before designing anything, and a greenfield crate
under that name would have collided with it at publish time.

Reading the published 0.3.5 source rather than its description showed the
diagnosis above still holds, in the published kit:

- `publish` spawns each subscriber callback and then awaits every handle
  before returning (`for handle in handles { let _ = handle.await; }`), so
  publisher latency is still the slowest subscriber's latency. Concurrent,
  but not decoupled.
- `let _ = handle.await` discards the `JoinError`, so a panicking callback is
  silently swallowed — indistinguishable from a healthy one.
- No bounded queue, no overflow policy, no queue depth, no counters.

0.3.5 is otherwise strong (typed `EventBus<T>`, `*`/`**` wildcards, envelope
metadata, persistence with postgres/sqlite/memory stores and replay), and the
gap is squarely in the delivery layer.

## Decision

Evolve the existing kit rather than replace it, keeping the delivery layer in
the kit and the domain event in CivitForge.

### Shipped in eventbus-kit 0.4.0 (additive)

- **Bounded delivery** — `subscribe_bounded` gives a subscriber a queue of at
  most `capacity` envelopes plus a worker task. The publisher enqueues and
  returns, so a slow consumer cannot reach a producer's critical path.
- **Explicit overflow policy** — `DropOldest`, `DropNewest`, `Reject`,
  chosen per subscriber. Nothing is lost silently, and no policy blocks the
  publisher.
- **Observable saturation** — `stats()` exposes queue depth, delivered,
  dropped, rejected, and panicked counters. Queue depth is the early signal:
  a subscriber that cannot keep up is visible before `dropped` proves data
  was lost.
- **Producer-side reporting** — `publish_report` returns
  `DeliveryReport { matched, enqueued, dropped, rejected }`, so a producer can
  react to loss instead of discovering it later in a consumer's backlog.
- **Panic containment that counts** — a panicking callback increments
  `panicked` and the worker's delivery continues.

`subscribe`, `subscribe_sync`, `publish`, `publish_sequential`, and the
persistence APIs are unchanged, so 0.4.0 is a minor release.

### Migration

The first draft of this section planned to rewrite `EventBus` as a thin
adapter over the kit. Reading the code instead of counting call sites showed
that would have been wasted work:

- **`EventBus` is dead code in production.** Outside its own module it appears
  exactly three times: the `AppState::event_bus` field, a
  `WebSocketManager::event_bus` field that is never read (`#[allow(dead_code)]`),
  and its own tests. No request handler publishes through it. It is
  constructed, stored, and benched — never used.
- The `52 subscribe sites / 7 publish sites` in the Context section are counts
  of every `.subscribe(`/`.publish(` call in `civit-core`, not calls on this
  bus. They belong to `WebSocketManager::subscribe`, the log-stream broadcast,
  and the notification broadcaster.

So the plan is:

1. **Delete** the hand-rolled `EventBus` (247 lines) and its `AppState` and
   `WebSocketManager` plumbing. A subsystem that no production path reaches is
   a trap for the next reader, and it is what made this look like a live
   migration when it was not.
2. **Bound the fan-out that is actually live.** Unbounded
   `mpsc::UnboundedSender` fan-out appears in four modules:
   `events/websocket.rs` (per-connection), `realtime/channels.rs`,
   `realtime/collaboration.rs`, and `realtime/graphql_subscriptions.rs`. Each
   holds `Vec<UnboundedSender<...>>`, so one slow consumer grows its queue
   without limit and the only pressure signal is host memory exhaustion. This
   is the same unbounded-backlog failure the kit addresses, in the place where
   it can actually happen.
3. Adopt the kit's bounded semantics — an explicit per-subscriber capacity and
   overflow policy — for those four paths, and expose drop counters through
   the Prometheus endpoint alongside the existing HTTP instruments.

### Status

The kit work is complete and published (`eventbus-kit` 0.4.0, 59 tests,
clippy clean under `--all-features -D warnings`). Steps 1–3 are **not started**:
the build host has 1 GB of available RAM with swap fully consumed, and a
`civit-core` build was previously OOM-killed at 2.2 GB RSS. Editing four
modules that cannot be compiled or tested would trade a verified tree for an
unverified one, so the code is left untouched and green pending build capacity.

## Consequences

- Handlers that must complete before a response returns no longer do. The
  migration must audit which of the 52 sites actually depended on that.
- A subscriber that cannot keep up becomes *visible* (depth, drops) instead
  of implicitly throttling every publisher.
- Queue sizes and policies become per-subscriber decisions. Defaults are
  conservative: `DropOldest` for log and websocket fan-out, `Reject` for
  state-critical consumers that must surface loss to the producer.
- The in-memory replay ring stays for now. eventbus-kit's store-backed replay
  is a better fit for durable replay and is a follow-up, not a prerequisite.

## Alternatives rejected

- **A new greenfield kit.** Rejected: the crate already exists, is published,
  and is stronger than a rewrite in typing, wildcards, and persistence.
- **Adopt a broker (NATS/Kafka) for in-process events.** Rejected: the bus
  carries in-process domain events only; a broker adds an operational
  dependency for latency the process does not need.
- **Keep the synchronous bus and document the risk.** Rejected: the risk is a
  client-visible latency cliff, and "document it" is how the hand-rolled
  counters hid for so long.
- **Extract the whole events module (bus + websocket + scaler).** Rejected as
  too broad for one change: the WebSocket layer carries connection lifecycle
  and fan-out policy that is not delivery semantics. Transport first, sockets
  later, once the transport is proven.
