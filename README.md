# Aper

[![crates.io](https://img.shields.io/crates/v/aper.svg)](https://crates.io/crates/aper)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/aper/)
[![docs.rs](https://img.shields.io/badge/docs-latest-orange)](https://aper.dev/doc/aper/index.html)
![wokflow state](https://github.com/aper-dev/aper/workflows/build/badge.svg)

Aper is a framework for real-time sharing of application state
over [WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
Its use cases include browser-based collaboration tools and
multiplayer in-browser games.

Specifically, Aper provides scaffolding to represent your program as a state
machine, as well as the infrastructure to keep that state machine synchronized
across multiple instances running in your users' browsers.

Aper integrates with [Yew](https://yew.rs/docs/en/) on the client side, and
[Actix](https://actix.rs/) for the server. Although the focus is on browser-based
apps running in WebAssembly and communicating over WebSocket, the core state
machine scaffolding can be used independent of the client/server architecture,
and even with non-WebSocket protocols.

**Aper is rapidly evolving. Consider this a *technology preview*.**

- [Documentation](https://docs.rs/aper/)
- [Getting Started with Aper guide](https://aper.dev/guide/)
- [Redwords](https://redwords.paulbutler.org), an experimental multiplayer word game built with Aper.

## Roadmap

Before the first non-preview release, the following need to be sorted out:

- [x] State transitions that can occur with no user input (e.g. for a timer in a game).
- [ ] Optimistic state updates on the client, with rollback if necessary.
- [ ] Implement graceful reconnection in the client. (e.g. iOS seems to drop
      websocket connections of background tabs, need to auto-reconnect)
- [ ] Allow the state machine to handle disconnection.
- [ ] Allow the state machine to “reject” a transition instead of just treating it
      as a no-op, in order to avoid propagating it.
- [x] Use a factory pattern to produce state machines rather than a no-argument
      `new` function, for flexibility.
- [ ] Add turn-key “channel creation” UI.
- [x] The server should allow binary or text connections, and the client should switch between
      json and bincode depending on whether it has the development flag.

The immediate roadmap has a strong emphasis on figuring out the right interface
between Aper and application code. Once that's sorted out, longer-term tasks can
focus on scaling Aper up to a production environment:

- Add a separate concept of “player state” in addition to game state. Player state
  includes things like name or cursor position, but cannot be used for state updates,
  and as a result can be sent out-of-order.
- Make state machines more composable.
- Integrating with authentication/permissions. I don't plan for Aper to ever
  be opinionated about an auth framework, but it needs to provide hooks to
  allow it to integrate with other systems.
- Journaled state storage, both indefinitely for state persistence and
  short-term to enable graceful node failure.
- Load balancing rooms between multiple servers.
- “Agents” that are spun up by the server that have the same interface to the
  state machine that users do, but do not have a human attached. These could
  allow a way to access external resources, non-deterministic computation, etc.
  in a way that does not break the restriction that state updates are
  deterministic.
- Pre-built data structures like lists, trees, and sets, and a derive macro
  to turn any struct built with them into a state machine.
- Pre-built string data structure implementing Operational Transform.
