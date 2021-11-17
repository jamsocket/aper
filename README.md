# [Aper](https://aper.dev)

[![crates.io](https://img.shields.io/crates/v/aper.svg)](https://crates.io/crates/aper)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/aper/)
[![docs.rs](https://img.shields.io/badge/docs-latest-orange)](https://aper.dev/doc/aper/index.html)
[![wokflow state](https://github.com/aper-dev/aper/workflows/build/badge.svg)](https://github.com/aper-dev/aper/actions/workflows/rust.yml)

<img src="ape.svg" alt="Cartoonized face of an ape." width="200px" />

Aper is a framework for real-time sharing of application state
over [WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
Its use cases include browser-based collaboration tools and
multiplayer in-browser games.

Specifically, Aper provides scaffolding to represent your program as a state
machine, as well as the infrastructure to keep that state machine synchronized
across multiple instances running in your users' browsers.

Aper integrates with [Yew](https://yew.rs/docs/en/) on the client side, and
[Jamsocket](https://github.com/jamsocket/jamsocket) as the server.
Although the focus is on browser-based apps running in WebAssembly and communicating
over WebSocket, the core state machine scaffolding can be used independent of the
client/server architecture, and even with non-WebSocket protocols.

**Aper is rapidly evolving. Consider this a *technology preview*.** See the [list of issues outstanding for version 1.0](https://github.com/aper-dev/aper/labels/v1-milestone)

- [Documentation](https://docs.rs/aper/)
- [Getting Started with Aper guide](https://aper.dev/guide/)
- [Examples](examples)
- [Talk on Aper for Rust Berlin (20 minute video)](https://www.youtube.com/watch?v=HNzeouj0eKc&t=1832s)
