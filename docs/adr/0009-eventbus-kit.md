# ADR-0009: eventbus-kit: bounded delivery for the event bus
- 2,514 lines across bus, websocket, log_stream, websocket_scaler; as the
  handler; a client-visible latency cliff.
  silently swallowed; indistinguishable from a healthy one.
- **Bounded delivery**; `subscribe_bounded` gives a subscriber a queue of at
- **Explicit overflow policy**; `DropOldest`, `DropNewest`, `Reject`,
- **Observable saturation**; `stats()` exposes queue depth, delivered,
- **Producer-side reporting**; `publish_report` returns
- **Panic containment that counts**; a panicking callback increments
  constructed, stored, and benched; never used.
   - `events/websocket.rs`; **live**. Per-connection outbound queues, and
     `realtime/graphql_subscriptions.rs`; **not wired**. `realtime::`
   is pruned from the fan-out list, which `channels.rs` never did; it
3. It called `events/websocket.rs` live because the router reaches it -
absent from that scrape, correctly; OTel counters appear on first