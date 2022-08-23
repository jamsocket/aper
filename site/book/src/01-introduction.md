# Introduction

**Aper** is a Rust library for representing data that can be read and written to by multiple users in real time.

Use-cases of Aper include managing the state of an application with real-time collaboration features, creating
an timestamped audit trail of an arbitrary data structure, and synchronizing the game state of a multiplayer
game.

The core `aper` library implements the underlying data structures and algorithms, but is agnostic to the
actual mechanism for transfering data on a network. The crates `aper-yew` and `aper-serve` provide a client
and server implementation aimed at synchronizing state across multiple `WebAssembly` clients using `WebSockets`.

## How it works

For Aper to synchronize state, it must be represented as a **state 
machine**. This means that:
1. It implements the `StateMachine` trait, which has two type arguments (`Transition` and `Conflict`) and one method: `apply(&self, t: &Transition)`.
2. **All** changes to its internal state flow through this `apply` method.
3. Updates to state are entirely deterministic. They may depend on the current state and any data
   that is contained in the transition value, and nothing else.
4. If a conflict arises (i.e. if `apply` returns anything other than `Ok(())`), the state machine is not
   mutated.

#1 is enforced by Rust's type system, but it's your responsibility to satisfy the other three. In particular,
accessing the current time, non-determistic random number generators, or external data in `apply` is
a violation of #3.

### Keeping State in Sync

In principle, the way that Aper keeps state in sync is pretty simple: when a client connects, they receive a
full copy of the latest copy of the state. Thereafter, they receive a real-time stream of `Transition`s. Every
client receives the same transitions in the same order, so their states are updated in lockstep. **This is why
it's important that `apply` is deterministic.** If it were not, states could become divergent even if they
received the same transition stream.

Note that for this model to work, the client can't throw away previous states immediately when a local transition
happens, because the server might accept a transition from another peer before accepting the one created locally.
We need the old state in order to replay the transitions in the right order. In order to do this, the client
keeps the entire chain of transitions from the last transition confirm by the server up to the most recent local
transition.
