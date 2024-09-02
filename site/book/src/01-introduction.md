# Introduction

**Aper** is a Rust data synchronization library. Fundamentally, Aper lets you create a `struct` that can be synchronized across multiple instances of your program, possibly running across a network.

Use-cases of Aper include:
- managing the state of an application with real-time collaboration features
- creating an timestamped audit trail of an arbitrary data structure
- synchronizing the game state of a multiplayer game.

The core `Aper` library is not tied to a particular transport, but works nicely with WebSocket. The `aper-websocket-client` and `aper-serve` crates define WebAssembly-based client and server libraries for Aper data structures.

## Design Goals

Aper is designed for **server-authoritative** synchronization, where one instance of an Aper program is considered the “server”, and others are ”clients”.

This is in contrast to conflict-free replicated data types (CRDTs), which are designed to work in peer-to-peer environments. A design goal of Aper is to allow developers to take full advantage of the server authority, which makes it possible to enforce data invariants.

The other guiding goals of Aper are:

- Local (optimistic) updates should be fast.
- Data synchronization concerns should not live in application code.

Aper is designed for structured/nested data. It is not optimized for long, flat sequences like text.

## Overview

Aper provides a number of traits and structs that are key to understanding Aper.

The **`Store`** struct is the core data store in Aper. Aper knows how to synchronize a `Store` *one-way* across a network, i.e. from the server to clients.

The **`AperSync`** trait designates a struct that expects to be stored in a `Store`. An `AperSync` struct is really just a reference into some data in the store, along with associated methods for interpreting it as Rust types.

Since a `Store` can be synchronized by Aper, and `AperSync` is just a reference to data in a `Store`, `AperSync` types can be synchronized.

But that synchronization is only one-way: from server to clients. Generally, clients will also want to modify the data, which is where the `Aper` trait comes in.

The **`Aper`** trait designates a struct as being *bidirectionally* synchronizable. It defines a set of actions (called *intents*) that can be performed on the store to update it.

**`AperClient`** and **`AperServer`** provide a “sans-I/O” client/server sync protocol, implemented for the client- and server-side respectively. Typically, you will not use them directly from application code, but instead use crates like `aper-websocket-client` that use them in combination with a particular I/O library.
