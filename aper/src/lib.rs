#![allow(unused)]

mod aper;
pub mod connection;
pub mod data_structures;
mod treemap;

pub use aper::*;
pub use aper_derive::Attach;
use serde::{Deserialize, Serialize};
pub use treemap::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Mutation {
    pub prefix: Vec<Bytes>,
    pub entries: Vec<(Bytes, Option<Bytes>)>,
}

pub type Bytes = Vec<u8>;
