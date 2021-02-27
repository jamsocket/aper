# Introduction

Aper is a framework that lets you synchronize program state between users. In Aper, all state is represented as a **state machine**, which means that:
1. It implements the `StateMachine` trait, which has one method: `apply`.
2. **All** changes to state flow through the `apply` method, which takes a single argument
   (the type of that argument is up to you).
3. Updates to state are entirely deterministic. They may depend on the current state and any data
   that is contained in the transition value, and nothing else.

#1 is enforced by Rust's type system, but it's your responsibility to satisfy #2 and #3. In particular,
accessing the current time, non-determistic random number generators, or external data in `apply` is
a violation of #3.

## Keeping State in Sync

In principal, the way that Aper keeps state in sync is pretty simple: when a client connects, they receive a full copy of the latest copy of the state. Thereafter, they receive a real-time stream of `Transition`s. Every client receives the same transitions in the same order, so their states are updated in lockstep. **This is why it's important that `apply` is deterministic.** If it were not, states could become divergent even if they received the same transition stream.

Note that for this model to work, the client can't immediately apply a transition to its local state, because the client doesn't know whether another client is sending the server a transition at the same time. The client has two options:

- Accept this latency. Depending on the use-case, a few hundred milliseconds of latency might be tolerable and the simpler model is easier to reason about.
- Keep two copies of the state, called `optimist` and `pessimist`, as well as a FIFO queue `stash` of local transitions. As local transitions fire, push them to `stash` and proactively apply them to `optimist`. `optimist` is used to render the view, so local transitions appear automatically. When the server sends a transition, check if it is the next transition in our `stash`. If so, pop it from the `stash` and apply it to `pessimist`. Otherwise, we apply the server-sent transition to `pessimist`, then clone it and apply every transition in the `stash` to the clone. This clone becomes the new value of `optimist`, and the old value is discarded.

Aper implements both of these approaches.