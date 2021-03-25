use yourcontrols_types::{Error, Result, VarId};

/// The data size of the ClientDataArea.
pub const DATA_SIZE: usize = 8192;
/// A result acception any error.
pub type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn map_ids<T: Clone>(map: &Vec<T>, ids: Vec<VarId>) -> Result<Vec<T>> {
    let mut vars = Vec::new();
    for id in ids {
        vars.push(map.get(id).ok_or(Error::None)?.clone())
    }
    Ok(vars)
}
