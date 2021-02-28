use crate::{StateMachine, Transition};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A struct that can wrap a value so that it can be used in place
/// of a state machine, but
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Constant<T: Clone + PartialEq + Debug + Unpin> {
    value: T,
}

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug>
    Constant<T>
{
    /// Create a new [Atom] with a given initial value.
    pub fn new(initial: T) -> Self {
        Constant { value: initial }
    }

    /// Retrieve the current value of the atom.
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: 'static + Serialize + DeserializeOwned + Unpin + Send + Clone + PartialEq + Debug>
    StateMachine for Constant<T>
{
    type Transition = InvalidTransition;

    fn apply(&mut self, _transition_event: InvalidTransition) {
        panic!("Constant should never receive transition event.");
    }
}

/// Represents a transition used to change the value of an [Atom].
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct InvalidTransition;

impl Transition for InvalidTransition {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace() {
        let constant = Constant::new(5);
        assert_eq!(5, *constant.value());
    }
}
