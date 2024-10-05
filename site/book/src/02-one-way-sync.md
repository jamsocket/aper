## One-Way Synchronization

`AperSync` means that the struct can be synchronized *unidirectionally*. An `AperSync` struct does not own its own data; instead, its fields are references into a `Store`. `Store` is a hierarchical map data structure provided by Aper that can be synchronized across a network.

Typically, you will not implement `AperSync` directly, but instead derive it. For example, here's a simple `AperSync` struct that could represent an item in a to-do list:

```rust
use aper::{AperSync, data_structures::Atom};

#[derive(AperSync, Clone)]
struct ToDoItem {
   done: Atom<bool>,
   name: Atom<String>,
}
```

In order to derive `AperSync`, **every field must implement AperSync**. Typically, this means that fields will either be data structures imported from the `aper::data_structures::*` module, or `structs` that you have derived `AperSync` on.

`Atom` is the most basic `AperSync` type; it represents an atomic value with the provided type. Any serde-serializable type can be used, but keep in mind that these values are opaque to the synchronization system and any modifications mean replacing them entirely.

Generally, for compound data structures, you should use more appropriate types. Here's an example of using `AtomMap`:

```rust
use aper::{AperSync, data_structures::AtomMap};

#[derive(AperSync, Clone)]
struct PhoneBook {
   name_to_number: AtomMap<String, String>,
}
```

The `Atom` in `AtomMap` refers to the fact that the **values** of the map act like `Atom`s: they do not need to implement `AperSync`, but must be (de)serializable.

Aper also provides a type of map where values are `AperSync`. This allows more fine-grained updates to the data structure. For example, you might want to create a todo list by mapping a unique ID to a `ToDoItem`:

```rust
use aper::{AperSync, data_structures::{Atom, Map}};
use uuid::Uuid;

#[derive(AperSync, Clone)]
struct ToDoItem {
   pub done: Atom<bool>,
   pub name: Atom<String>,
}

#[derive(AperSync, Clone)]
struct ToDoList {
   pub items: Map<Uuid, ToDoItem>,
}
```

## Using `AperSync` types

`AperSync` structs are constructed by “attaching” them to a `Store`. Every `AperSync` type implicitly has a default
value, which is what you get when you attach it to an empty `Store`.

When modifying collections of `AperSync` like `Map`, you don't insert new values directly. Instead, you call a method like
`get_or_create` that creates the value as its default, and then call mutators on the value that is returned, like so:

```rust
# use aper::{data_structures::{Atom, Map}};
# use uuid::Uuid;
use aper::{AperSync, Store};

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

fn main() {
   let store = Store::default();
   let mut todos = ToDoList::attach(store.handle());

   let mut todo1 = todos.items.get_or_create(&Uuid::new_v4());
   todo1.name.set("Do laundry".to_string());

   let mut todo2 = todos.items.get_or_create(&Uuid::new_v4());
   todo2.name.set("Wash dishes".to_string());
   todo2.done.set(true);
}
```
