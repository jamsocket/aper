//! Implementations of data structures built on [`crate::StateMachine`].

mod atom;
mod atom_rc;
mod constant;
mod counter;
mod list;
mod map;

pub use atom::{Atom, ReplaceAtom};
pub use atom_rc::{AtomRc};
pub use constant::Constant;
pub use counter::Counter;
pub use list::{List, ListItem, ListOperation, OperationWithId};
pub use map::Map;
