## Bidirectional Synchronization

Synchronization in Aper is *asymmetric*, meaning that the method of synchronizing
the data structure from the server to the client is different from the method of
synchronizing from the client to the server.

The server sends changes to the client by telling it directly how to modify its
`Store`. These messages are called *mutations*.

For the client to send changes to the server, you need to supply two things:

- An *intent* type (usually an `enum`), that can represent actions that can be taken
on the data to modify it.
- An `apply` function that takes the data structure and an intent, and updates the
data structure accordingly.

Both of these, as well as an error type, are provided through the `Aper` trait.

### Example

In the last section, we implemented a `ToDoList`:

```rust
use aper::{AperSync, data_structures::{Atom, Map}};

#[derive(AperSync, Clone)]
struct ToDoItem {
   pub done: Atom<bool>,
   pub name: Atom<String>,
}

#[derive(AperSync, Clone)]
struct ToDoList {
   pub items: Map<String, ToDoItem>,
}
```

To create an **intent** type, we should consider the actions a user might take. A minimal set of intents (inspired by [TodoMVC](https://todomvc.com/)) is:

- Create a new task
- Change the name of an existing task
- Mark a task as done / not done
- Remove all completed tasks

In code, that looks like:

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

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
```

Note that when we take action on an existing task, we need a way to identify it,
so we give each task a universally unique identifier (UUID). This UUID is generated
on the client and sent as part of the `CreateTask` message.

This is a bit different from what you might expect if you're used to remote procedure
call (RPC) APIs, where the server is responsible for generating IDs. It's important
here because the client may need to create an intent that refers to a task before it
hears back from the server with an ID (for example, if the network is interrupted
or the user has gone offline.)

Now, we implement `Aper` for `ToDoList`:

```rust
# use aper::{AperSync, data_structures::{Atom, Map}, IntentMetadata};
# use serde::{Serialize, Deserialize};
# use uuid::Uuid;
# 
# #[derive(AperSync, Clone)]
# struct ToDoItem {
#    pub done: Atom<bool>,
#    pub name: Atom<String>,
# }
# 
# #[derive(AperSync, Clone)]
# struct ToDoList {
#    pub items: Map<Uuid, ToDoItem>,
# }
# 
# #[derive(Serialize, Deserialize, Clone, std::cmp::PartialEq)]
# enum ToDoIntent {
#     CreateTask {
#         id: Uuid,
#         name: String,
#     },
#     RenameTask {
#         id: Uuid,
#         name: String,
#     },
#     MarkDone {
#         id: Uuid,
#         done: bool,
#     },
#     RemoveCompleted,
# }

use aper::Aper;

impl Aper for ToDoList {
    type Intent = ToDoIntent;
    type Error = ();

    fn apply(&mut self, intent: &ToDoIntent, _metadata: &IntentMetadata) -> Result<(), ()> {
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
                // TODO: need some way to iterate from Map first!
            }
        }

        Ok(())
    }
}
```
