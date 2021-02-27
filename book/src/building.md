# Building a state machine

For example, let's say we have a counter (this is not a state machine):

```rust
struct Counter {value: i64}

impl Counter {
    pub fn add(&mut self, i: i64) {
        self.value += i;
    }

    pub fn subtract(&mut self, i: i64) {
        self.value -= i;
    }

    pub fn reset(&mut self, i: i64) {
        self.value = 0;
    }
}

# fn main() {}
```

We can rewrite this as a state machine like this:

```rust
# use aper::StateMachine;
# use serde::{Serialize, Deserialize};
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# struct Counter {value: i64}
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# enum CounterTransition {
#    Add(i64),
#    Subtract(i64),
#    Reset,
# }

impl StateMachine for Counter {
    type Transition = CounterTransition;

    fn apply(&mut self, event: CounterTransition) {
        match event {
            CounterTransition::Add(i) => {
                self.value += i;
            }
            CounterTransition::Subtract(i) => {
                self.value -= i;
            }
            CounterTransition::Reset => {
                self.value = 0;
            }
        }
    }
}
# fn main() {}
```


```rust
# use aper::StateMachine;
# use serde::{Serialize, Deserialize};
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# struct Counter {value: i64}
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# enum CounterTransition {
#    Add(i64),
#    Subtract(i64),
#    Reset,
# }

impl StateMachine for Counter {
    type Transition = CounterTransition;

    fn apply(&mut self, event: CounterTransition) {
        match event {
            CounterTransition::Add(i) => {
                self.value += i;
            }
            CounterTransition::Subtract(i) => {
                self.value -= i;
            }
            CounterTransition::Reset => {
                self.value = 0;
            }
        }
    }
}

# fn main() {}
```

Now, any attempt to modify the state of the counter must flow through `apply` as a `CounterTransition`. We could use `CounterTransition`'s constructors directly, but the idiomatic approach that Aper encourages is to implement methods with the same signatures as our original modifiers but that return the `Transition` type:

```rust
# use aper::StateMachine;
# use serde::{Serialize, Deserialize};
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# struct Counter {value: i64}
#
# #[derive(Serialize, Deserialize, Debug, Clone)]
# enum CounterTransition {
#    Add(i64),
#    Subtract(i64),
#    Reset,
# }

impl Counter {
    pub fn add(&self, i: i64) -> CounterTransition {
        CounterTransition::Add(i)
    }

    pub fn subtract(&self, i: i64) -> CounterTransition {
        CounterTransition::Subtract(i)
    }

    pub fn reset(&self, i: i64) -> CounterTransition {
        CounterTransition::Reset
    }
}
# fn main() {}
```

Notice how these no longer require a mutable reference to `self`, since they do not actually make any changes, they just return an object *representing* the change.