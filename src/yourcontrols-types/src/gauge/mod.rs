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
