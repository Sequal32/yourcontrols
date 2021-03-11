mod fragment;
mod messages;

pub use fragment::*;
pub use messages::*;

/// Type used for keeping track of vars mapped to values.
pub type DatumKey = u32;
/// Type used for getting/setting values.
pub type DatumValue = f64;
/// Time type used for interpolation.
pub type Time = f64;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum InterpolationType {
    Default,
    DefaultConstant,
    Wrap360,
    Wrap180,
    Wrap90,
    Invert,
    InvertConstant,
}

impl InterpolationType {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::InvertConstant | Self::DefaultConstant => true,
            _ => false,
        }
    }
}
