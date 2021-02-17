use serde::Deserialize;
use std::fmt::Debug;
use yew::format::{Binary, Bincode, Json, Text};

/// A format (conforming to the spec used by [yew::format]) that
/// dynamically interprets messages as either JSON or bincode, depending
/// on whether they are received as text or binary (respectively).
#[derive(Debug)]
pub struct WireWrapped<T: for<'de> Deserialize<'de>> {
    /// The value decoded from the wire.
    pub value: T,

    /// True if the message was received in binary.
    /// This is used to decide whether we should use text or binary to
    /// respond back to the server.
    pub binary: bool,
}

impl<T: for<'de> Deserialize<'de>> From<Text> for WireWrapped<T> {
    fn from(text: Text) -> Self {
        let j: Json<Result<T, _>> = text.into();
        WireWrapped {
            value: j.0.unwrap(),
            binary: false,
        }
    }
}

impl<T: for<'de> Deserialize<'de>> From<Binary> for WireWrapped<T> {
    fn from(bin: Binary) -> Self {
        let j: Bincode<Result<T, _>> = bin.into();
        WireWrapped {
            value: j.0.unwrap(),
            binary: true,
        }
    }
}
