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
//! For the purposes of Aper, a state machine is simply a `struct` or `enum` that
//! implements [StateMachine] and has the following properties:
//! - It defines a [StateMachine::Transition] type, through which every
//!   possible change to the state can be described. It is usually useful,
//!   though not required, that this be an `enum` type.
//! - All state updates are deterministic: if you clone a [StateMachine] and a
//!   [Transition], the result of applying the cloned transition to the cloned
//!   state must be identical to applying the original transition to the original
//!   state.
//!
//! Here's an example [StateMachine] implementing a counter:
//!
//! ```rust
//! # use aper::{StateMachine, Transition};
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize, Clone, Debug, Default)]
//! struct Counter(i64);
//!
//! #[derive(Transition, Serialize, Deserialize, Clone, Debug, PartialEq)]
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
//! ## State Programs
//!
//! [StateMachine]s can take whatever transition types they want, but in order to interface with
//! the Aper client/server infrastructure, a [StateMachine] must have a [TransitionEvent] transition
//! type. This wraps up a regular [Transition] with metadata that the client produces (namely, the
//! ID of the player who initiated the event and the timestamp of the event).
//!
//! In order to tell the Rust typesystem that a [StateMachine] is compatible, it must also implement
//! the [StateProgram] trait. This also gives you a way to implement *suspended events*.
//!
//! Typically, a program in Aper will have only one trait that implements [StateProgram], but may
//! have multiple traits that implement [StateMachine] used in the underlying representation of
//! [StateProgram].
//!
//! If you just want to serve a [StateMachine] data structure and don't need transition metadata,
//! you can construct a [StateMachineContainerProgram] which simply strips the metadata and passes
//! the raw transition into the state machine, i.e.:
//!
//! ```rust
//! # use aper::{StateMachine, Transition};
//! # use serde::{Serialize, Deserialize};
//! # #[derive(Serialize, Deserialize, Clone, Debug, Default)]
//! # struct Counter;
//! #
//! # #[derive(Transition, Serialize, Deserialize, Clone, Debug, PartialEq)]
//! # struct CounterTransition;
//! #
//! # impl StateMachine for Counter {
//! #     type Transition = CounterTransition;
//! #
//! #     fn apply(&mut self, event: CounterTransition) { unimplemented!() }
//! # }
//! #
//! # pub fn main() {
//! use aper::StateMachineContainerProgram;
//! let state_program = StateMachineContainerProgram(Counter::default());
//! # }
//! ```
//!
//! ## How it works
//!
//! When a client first connects to the server, the server sends back a complete serialized
//! copy of the current state. After that, it sends and receives only [TransitionEvent]s to/from
//! the server. By applying these [TransitionEvent]s to
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
//! ## Why not CRDT?
//!
//! [Conflict-free replicated data types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type)
//! are a really neat way of representing data that's shared between peers.
//! In order to avoid the need for a central “source of truth”, CRDTs require
//! that update operations (i.e. state transitions) be [commutative](https://en.wikipedia.org/wiki/Commutative_property).
//! This allows them to represent a bunch of common data structures, but doesn't
//! allow you to represent arbitrarily complex update logic.
//!
//! By relying on a central authority, a state-machine approach allows you to
//! implement data structures with arbitrary update logic, such as atomic moves
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
pub use state_machine::{StateMachine, Transition};
pub use state_program::{StateMachineContainerProgram, StateProgram, StateProgramFactory};

pub mod data_structures;
mod state_machine;
mod state_program;

/// An opaque identifier for a single connected user.
#[derive(Clone, Hash, Debug, PartialEq, Ord, PartialOrd, Eq, Serialize, Deserialize, Copy)]
pub struct PlayerID(pub usize);

impl Display for PlayerID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TransitionEvent<T>
where
    T: Transition + Clone,
{
    #[serde(with = "ts_milliseconds")]
    pub timestamp: Timestamp,
    pub player: Option<PlayerID>,
    #[serde(bound = "")]
    pub transition: T,
}

impl<T: Transition> TransitionEvent<T> {
    pub fn new(
        player: Option<PlayerID>,
        timestamp: Timestamp,
        transition: T,
    ) -> TransitionEvent<T> {
        TransitionEvent {
            player,
            timestamp,
            transition,
        }
    }
}

impl<T: Transition> Transition for TransitionEvent<T> {}

/// A message from the server to a client that tells it to update its state.
#[derive(Serialize, Deserialize, Debug)]
pub enum StateUpdateMessage<T: Transition, State: StateProgram<T>> {
    /// Instructs the client to completely discard its existing state and replace it
    /// with the provided one. This is currently only used to set the initial state
    /// when a client first connects.
    ReplaceState(
        #[serde(bound = "")] State,
        #[serde(with = "ts_milliseconds")] Timestamp,
        PlayerID,
    ),

    /// Instructs the client to apply the given [TransitionEvent] to its copy of
    /// the state to synchronize it with the server. All state updates
    /// after the initial state is sent are sent through [StateUpdateMessage::TransitionState].
    TransitionState(#[serde(bound = "")] TransitionEvent<T>),
}
