use serde::Deserialize;
use std::fmt::Debug;
use yew::format::{Binary, Bincode, Json, Text};

#[derive(Debug)]
pub struct WireWrapped<T: for<'de> Deserialize<'de>> {
    pub value: T,
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