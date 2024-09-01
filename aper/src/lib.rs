#![allow(unused, clippy::type_complexity)]

mod aper;
pub mod connection;
pub mod data_structures;
mod listener;
mod store;

pub use aper::*;
pub use aper_derive::AperSync;
use serde::{Deserialize, Serialize};
pub use store::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Mutation {
    pub prefix: Vec<Bytes>,
    pub entries: PrefixMap,
}

pub type Bytes = Vec<u8>;
