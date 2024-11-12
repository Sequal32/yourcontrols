use crate::definitions;

pub type DefinitionsState = std::sync::Mutex<DefinitionsWrapper>;

pub struct DefinitionsWrapper(pub definitions::Definitions);

unsafe impl Send for DefinitionsWrapper {}

impl Default for DefinitionsWrapper {
    fn default() -> Self {
        Self(definitions::Definitions::new())
    }
}
