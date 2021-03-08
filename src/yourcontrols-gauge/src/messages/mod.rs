// should probably move to its own crate
mod fragment;
pub use fragment::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Payloads {
    Test,
    Hello {
        hi: bool,
        no: String,
        something: f64,
    },
}
