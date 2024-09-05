#![allow(clippy::type_complexity)]

mod aper;
pub mod connection;
pub mod data_structures;
mod listener;
mod store;
pub use aper::*;
pub use aper_derive::AperSync;
pub use bytes::Bytes;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use store::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Mutation {
    pub prefix: Vec<Bytes>,
    pub entries: PrefixMap,
}

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IntentEvent<T> {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: Timestamp,
    pub client: Option<u32>,
    pub intent: T,
}

impl<T> IntentEvent<T> {
    pub fn new(client: Option<u32>, timestamp: Timestamp, intent: T) -> IntentEvent<T> {
        IntentEvent {
            timestamp,
            client,
            intent,
        }
    }
}
