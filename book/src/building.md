# Building a state machine

To solidify the concept of state machines, let's start with a simple
example. Here's a simple data structure representing a counter. It stores
an integer and gives us methods to modify it.

```rust,noplaypen
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

By inspecting the code, you can see that `Counter` satisfies 
[condition #3](introduction.md) of a state machine in Aper:
its updates are deterministic. It does *not*, however, satisfy 
conditions #1 and #2: it does not implement `StateMachine`, and 
methods other than `apply` mutate the state.

(By the way, a good way to check if #2 is satisfied is to look for 
which methods take `&mut self`. In an Aper state machine, **only** 
`apply` should need a mutable reference to `self`.)

We can turn `Counter` into a state machine like this:

```rust,noplaypen
use aper::StateMachine;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Counter {value: i64}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum CounterTransition {
   Add(i64),
   Subtract(i64),
   Reset,
}

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

Now, any attempt to modify the state of the counter must flow through 
`apply` as a `CounterTransition`. We could use `CounterTransition`'s 
constructors directly, but the idiomatic approach that Aper encourages 
is to implement methods with the same signatures as our original 
modifiers but that return the `Transition` type:

```rust,noplaypen
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
#
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

Notice how these no longer require a mutable reference to `self`, since they do not actually make any changes, they just return an object *representing* the change. In fact, in this case they don't
even *read* from `self`, but that would be allowed and comes in
handy when we deal with more complex update logic.

I started by showing you how to implement your own state machine 
because I wanted you to see that it isn't
scary, but implementing state machines from scratch isn't the only way 
to use Aper. In the next few sections,
I'll show you how to build state machines by composing together 
primitives that Aper provides.