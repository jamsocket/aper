//! # Aper
//!
//! Aper is a framework for real-time sharing of arbitrary application state
//! over [WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
//!
//! With Aper, you represent your program state as a state machine by
//! implementing the [StateMachine] trait. Aper then provides the
//! infrastructure to keep clones of this state synchronized across
//! multiple clients, including clients running in WebAssembly via
//! WebSockets.
//!
//! ## Organization
//!
//! The Aper project is divided over a number of crates. This crate,
//! `aper`, does not provide any functionality, it merely defines a spec
//! that is used by the other crates.
//!
//! The other crates, `aper-yew` and `aper-actix`, provide client and server
//! implementations (respectively).
//!
//! ## What is a state machine?
//!
//! For the purposes of Aper, a state machine is simply a `struct` that
//! implements [StateMachine] and has the following properties:
//! - It defines a [StateMachine::Transition] type, through which every
//!   possible change to the state can be described. It is usually useful,
//!   though not required, that this be an `enum` type.
//! - All state updates are deterministic: if you clone a [StateMachine] and a
//!   [TransitionEvent] (which represents a [StateMachine::Transition] wrapped in some
//!   metadata), the result of applying that transition **must always** result
//!   in the same underlying state.
//!
//! This is similar to the state representation in frameworks like
//! [Redux](https://redux.js.org) and [Yew](https://yew.rs).
//!
//! ## How it works
//!
//! When a client connects to the server, it receives a complete serialized
//! copy of the current state. After that, it sends [StateMachine::Transition]s to the server
//! and receives only [TransitionEvent]s. By applying these [TransitionEvent]s to
//! its local copy, each client keeps its local copy of the state synchronized
//! with the server.
//!
//! It is important that the server guarantees that each client receives
//! [TransitionEvent]s in the same order, since the way a transition is applied
//! may depend on previous state. For example, if a transition pushes a value
//! to the end of a list, two clients receiving the transitions in a different
//! order would have internal states which represented different orders of the
//! list.
//!
//! ```rust
//! # use aper::{StateMachine};
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct Counter(i64);
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! enum CounterTransition {
//!     Reset,
//!     Increment(i64),
//!     Decrement(i64),
//! }
//!
//! impl StateMachine for Counter {
//!     type Transition = CounterTransition;
//!
//!     fn apply(&mut self, event: CounterTransition) {
//!         match event {
//!             CounterTransition::Reset => { self.0 = 0 }
//!             CounterTransition::Increment(amount) => { self.0 += amount }
//!             CounterTransition::Decrement(amount) => { self.0 -= amount }
//!         }
//!     }
//! }
//! ```
//!
//! ## Why not CRDT?
//!
//! [Conflict-free replicated data types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type)
//! are a really neat way of representing data that's shared between peers.
//! In order to avoid the need for a central “source of truth”, CRDTs require
//! that update operations (i.e. state transitions) be [commutative](https://en.wikipedia.org/wiki/Commutative_property).
//! This allows them to represent a bunch of common data structures, but doesn't
//! allow you to represent complex update logic.
//!
//! By relying on a central authority, a state-machine approach allows you to
//! implement data structures with arbitrary update logic, such as automic moves
//! of a value between two data structures, or the rules of a board game.
//!
//! ## Vocabulary Conventions
//!
//! - A **player** is a connection to the service. The term _user_ is probably a more conventional
//!   description, but _multiplayer_ is often used in the context of non-game multi-user apps, and
//!   I've chosen to adopt it here because I think our users should be having fun.
//! - A **transition** represents a way to update the state. For example, “draw a circle at (4, 6)”
//!   is a transition.
//! - An **event** (or *transition event*) is a specific invocation of a transition by a user
//!   at a time. For example, “player A drew a circle at (4, 6) at 10:04 PM” is an event.
//!   State machines *may* use the "who" and "when" data to determine how a transition is applied.
//!   Events may also not have a player associated with them (as in suspended events), but always
//!   have a time.
//! - A **channel** is the combination of a state object and the players currently connected to
//!   it. You can think of this as analogous to a room or channel in a chat app, except
//!   that the state is an arbitrary state machine instead of a sequential list of messages.
//!   The state of each channel is independent from one another: state changes in one channel
//!   do not impact the state in another, much like messages in one chat room do not appear in
//!   another.

use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};
pub use state_machine::{StateMachine, StateMachineFactory};
pub use suspended_event::SuspendedEvent;

pub mod data_structures;
mod state_machine;
mod suspended_event;

/// An opaque identifier for a single connected user.
#[derive(Clone, Hash, Debug, PartialEq, Ord, PartialOrd, Eq, Serialize, Deserialize, Copy)]
pub struct PlayerID(pub usize);

impl Display for PlayerID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A message from the server to a client that tells it to update its state.
#[derive(Serialize, Deserialize, Debug)]
pub enum StateUpdateMessage<State: StateMachine> {
    /// Instructs the client to completely discard its existing state and replace it
    /// with the provided one. This is currently only used to set the initial state
    /// when a client first connects.
    ReplaceState(
        #[serde(bound = "")] State,
        #[serde(with = "ts_milliseconds")] DateTime<Utc>,
        PlayerID,
    ),

    /// Instructs the client to apply the given [TransitionEvent] to its copy of
    /// the state to synchronize it with the server. Currently, all state updates
    /// after the initial state is sent are sent through [StateUpdateMessage::TransitionState].
    TransitionState(#[serde(bound = "")] State::Transition),
}
