//! Implementations of data structures built on [`crate::StateMachine`].

mod atom;
mod constant;
mod list;

pub use atom::{Atom, ReplaceAtom};
pub use constant::Constant;
pub use list::{List, ListItem, ListOperation, OperationWithId};
