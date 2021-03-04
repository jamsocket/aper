---
layout: default.liquid
title: Aper
---
<div style="background: #ffffba;
    margin-bottom: 30px;
    padding: 15px;
    line-height: 121%;
    border: 1px dashed black;">
<strong>Aper is very early stage.</strong> Feel free to poke around but please be aware that examples may not be in sync with the repo or crates versions of Aper.
</div>

<div id="header">
    <div id="text">Aper is like Git for your data structures.</div>
    <div id="image">
        <img src="ape.svg" style="width: 180px; height: 180px;" />
    </div>
</div>
<p class="explanation">Aper is an <a href="https://github.com/aper-dev/aper/blob/main/LICENSE">MIT-licensed</a> Rust library.</p>

<div class="buttons">
<a class="button primary" href="/guide/">Getting Started Guide</a>
<a class="button" href="https://github.com/aper-dev/aper">
<svg style="display: inline; width: 13px; fill: white;" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>GitHub icon</title><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>
GitHub</a>
<a class="button" href="/doc/">
<svg xmlns="http://www.w3.org/2000/svg" style="display: inline; width: 13px; fill: white;" viewBox="0 0 512 512"><path d="M488.6 250.2L392 214V105.5c0-15-9.3-28.4-23.4-33.7l-100-37.5c-8.1-3.1-17.1-3.1-25.3 0l-100 37.5c-14.1 5.3-23.4 18.7-23.4 33.7V214l-96.6 36.2C9.3 255.5 0 268.9 0 283.9V394c0 13.6 7.7 26.1 19.9 32.2l100 50c10.1 5.1 22.1 5.1 32.2 0l103.9-52 103.9 52c10.1 5.1 22.1 5.1 32.2 0l100-50c12.2-6.1 19.9-18.6 19.9-32.2V283.9c0-15-9.3-28.4-23.4-33.7zM358 214.8l-85 31.9v-68.2l85-37v73.3zM154 104.1l102-38.2 102 38.2v.6l-102 41.4-102-41.4v-.6zm84 291.1l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6zm240 112l-85 42.5v-79.1l85-38.8v75.4zm0-112l-102 41.4-102-41.4v-.6l102-38.2 102 38.2v.6z"></path></svg>
docs</a>
<a class="button" href="https://crates.io/crates/aper">crates.io</a>
</div>

<div class="code-caption">
    <p>Every data mutation is a <strong>first-class value</strong>.</p>
    <p>Serialize them to synchronize state across a network, or to create an audit log.</p>
</div>

```rust
use aper::{List, Atom};
// `List` represents an ordered list.
// `Atom` wraps a value to make it immutable
// except by replacement.

fn main() {
    let mut my_list: List<Atom<String>> = List::new();
    
    let (_id, transition) = my_list.push(Atom::new(
        "Hello Aper".to_string()));

    // `transition` represents the action of adding
    // "Hello Aper" to the list, but doesn’t actually
    // modify the data.

    my_list.apply(transition);

    // Now the transition is applied.
}
```

<br clear="both" />

<div class="code-caption">
    <p>Mutations can be applied <strong>out-of-order</strong>.</p>
    <p>Mutations encode intent, so concurrent mutations are cleanly applied where possible.</p>
</div>

```rust
use aper::{List, Atom};

fn main() {
    let mut my_list: List<Atom<u32>> = List::new();
    
    let (id1, transition1) = my_list.push(Atom::new(1));
    let (id2, transition2) = my_list.push(Atom::new(2));

    my_list.apply(transition2); // my_list = [2]
    my_list.apply(transition1); // my_list = [2, 1]

    let (_id3, transition3) = my_list.insert_between(id2, id1,
        Atom::new(3)
    );
    let (_id4, transition4) = my_list.insert_between(id2, id1,
        Atom::new(4)
    );

    my_list.apply(transition4); // my_list = [2, 4, 1]
    my_list.apply(transition3); // my_list = [2, 4, 3, 1]
}
```

<br clear="both" />

<div class="code-caption">
    <p>Implement your own <strong>update logic</strong>.</p>
    <p>Define your own units of state that integrate seamlessly with Aper's built-in data structures.</p>
</div>

```rust
use aper::{StateMachine, Transition};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Counter {value: i64}

#[derive(Transition, Serialize, Deserialize, Debug, Clone,
    PartialEq)]
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
```