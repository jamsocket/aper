# [Aper](https://aper.dev)

[![GitHub Repo stars](https://img.shields.io/github/stars/drifting-in-space/aper?style=social)](https://github.com/drifting-in-space/aper)
[![crates.io](https://img.shields.io/crates/v/aper.svg)](https://crates.io/crates/aper)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/aper/)
[![wokflow state](https://github.com/drifting-in-space/aper/workflows/build/badge.svg)](https://github.com/drifting-in-space/aper/actions/workflows/rust.yml)

Aper is a Rust library for data synchronization over a network.

Aper supports optimistic updates and arbitrary business logic, making it useful for real-time collabrative and agentic use cases.

## Introduction

(More docs coming soon)

Types marked with the `AperSync` trait can be stored in the `Store`, Aper's synchronizable data store.
Aper includes several data structures that implement `AperSync` in the `aper::data_structures` module, which
can be used as building blocks to build your own synchronizable types.

You can use these, along with the `AperSync` derive macro, to compose structs that also implement `AperSync`.

```rust
use aper::{AperSync, data_structures::{Atom, Map}};
use uuid::Uuid;

#[derive(AperSync)]
struct ToDoItem {
   pub done: Atom<bool>,
   pub name: Atom<String>,
}

#[derive(AperSync)]
struct ToDoList {
   pub items: Map<Uuid, ToDoItem>,
}
```

To synchronize from the server to clients, Aper replicates changes to the `Store` when it receives them. To synchronize
from clients to servers, we instead send *intents* to the server.

Intents are represented as a serializable `enum` representing every possible action a user might take on the data.
For example, in our to-do list, that represents creating a task, renaming a task, marking a task as (not) done, or
removing completed items.

```rust
use aper::Aper;

#[derive(Serialize, Deserialize, Clone, std::cmp::PartialEq)]
enum ToDoIntent {
    CreateTask {
        id: Uuid,
        name: String,
    },
    RenameTask {
        id: Uuid,
        name: String,
    },
    MarkDone {
        id: Uuid,
        done: bool,
    },
    RemoveCompleted,
}

impl Aper for ToDoList {
    type Intent = ToDoIntent;
    type Error = ();

    fn apply(&mut self, intent: &ToDoIntent) -> Result<(), ()> {
        match intent {
            ToDoIntent::CreateTask { id, name } => {
                let mut item = self.items.get_or_create(id);
                item.name.set(name.to_string());
                item.done.set(false);
            },
            ToDoIntent::RenameTask { id, name } => {
                // Unlike CreateTask, we bail early with an `Err` if
                // the item doesn't exist. Most likely, the server has
                // seen a `RemoveCompleted` that removed the item, but
                // a client attempted to rename it before the removal
                // was synced to it.
                let mut item = self.items.get(id).ok_or(())?;
                item.name.set(name.to_string());
            }
            ToDoIntent::MarkDone { id, done } => {
                let mut item = self.items.get(id).ok_or(())?;
                item.done.set(*done);
            }
            ToDoIntent::RemoveCompleted => {
                // TODO: need to implement .iter() on Map first.
            }
        }

        Ok(())
    }
}
```

---

**Aper is rapidly evolving. Consider this a technology preview.** See the [list of issues outstanding for version 1.0](https://github.com/drifting-in-space/aper/labels/v1-milestone)

- [Documentation](https://docs.rs/aper/)
- [Examples](https://github.com/drifting-in-space/aper/tree/main/examples)
- [Talk on Aper for Rust Berlin (20 minute video)](https://www.youtube.com/watch?v=HNzeouj0eKc&t=1852s)
