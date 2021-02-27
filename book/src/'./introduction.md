# Introduction

Aper is a framework that lets you synchronize program state between users. In Aper, all state is represented as a **state machine**. All it really means for something to be a state machine is that it implements the `StateMachine` trait, and that all state changes flow through its `apply` method.  `apply` takes a single argument of whichever type you like, so we typically use an `enum` to squeeze all desired update events into one type.

Calls to `apply` must be **deterministic**, i.e. a given transition applied to the same state should always result in the same new state. In particular, this requires care when implementing transitions involving randomness or the current time.