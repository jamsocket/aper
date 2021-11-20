# [Aper](https://aper.dev)

[![GitHub Repo stars](https://img.shields.io/github/stars/drifting-in-space/aper?style=social)](https://github.com/drifting-in-space/aper)
[![crates.io](https://img.shields.io/crates/v/aper.svg)](https://crates.io/crates/aper)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/aper/)
[![wokflow state](https://github.com/drifting-in-space/aper/workflows/build/badge.svg)](https://github.com/drifting-in-space/aper/actions/workflows/rust.yml)

<img src="https://aper.dev/ape.svg" alt="Cartoonized face of an ape." width="200px" />

Aper is a data structure library in which every data structure is a **state
machine**, and every mutation is a first-class value that can be serialized
and sent over the network, or stored for later.

## What is a state machine?

For the purposes of Aper, a state machine is simply a `struct` or `enum` that
implements `StateMachine` and has the following properties:
- It defines a `StateMachine::Transition` type, through which every
  possible change to the state can be described. It is usually useful,
  though not required, that this be an `enum` type.
- It defines a `StateMachine::Conflict` type, which describes a conflict which
  may occur when a transition is applied that is not valid at the time it is
  applied. For simple types where a conflict is impossible, you can use
  `NeverConflict` for this.
- All state updates are deterministic: if you clone a `StateMachine` and a
  `Transition`, the result of applying the cloned transition to the cloned
  state must be identical to applying the original transition to the original
  state.

Here's an example `StateMachine` implementing a counter:

```rust
use aper::{StateMachine, NeverConflict};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Counter { value: i64 };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum CounterTransition {
    Reset,
    Increment(i64),
    Decrement(i64),
}

impl StateMachine for Counter {
    type Transition = CounterTransition;
    type Conflict = NeverConflict;

    fn apply(&mut self, event: CounterTransition) -> Result<(), NeverConflict> {
        match event {
            CounterTransition::Reset => { self.value = 0 }
            CounterTransition::Increment(amount) => { self.value += amount }
            CounterTransition::Decrement(amount) => { self.value -= amount }
        }

        Ok(())
    }
}
```

## Why not CRDT?
[Conflict-free replicated data types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type)
are a really neat way of representing data that's shared between peers.
In order to avoid the need for a central “source of truth”, CRDTs require
that update operations (i.e. state transitions) be [commutative](https://en.wikipedia.org/wiki/Commutative_property).
This allows them to represent a bunch of common data structures, but doesn't
allow you to represent arbitrarily complex update logic.
By relying on a central authority, a state-machine approach allows you to
implement data structures with arbitrary update logic, such as atomic moves
of a value between two data structures, or the rules of a board game.

---

**Aper is rapidly evolving. Consider this a technology preview.** See the [list of issues outstanding for version 1.0](https://github.com/drifting-in-space/aper/labels/v1-milestone)

- [Documentation](https://docs.rs/aper/)
- [Getting Started with Aper guide](https://aper.dev/guide/)
- [Examples](https://github.com/drifting-in-space-in-space/aper/tree/main/examples)
- [Talk on Aper for Rust Berlin (20 minute video)](https://www.youtube.com/watch?v=HNzeouj0eKc&t=1852s)
