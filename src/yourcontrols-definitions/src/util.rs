use serde::{Deserialize, Serialize};
use yourcontrols_types::{ConditionMessage, InterpolationType, MappingType, VarType};

#[derive(Deserialize, Default, Debug, Clone)]
pub struct YamlTopDown {
    pub ignore: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub server: Option<Vec<String>>,
    pub shared: Option<Vec<String>>,
    pub init: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mapping {
    pub mapping: MappingType<String>,
    pub var: VarType,
    #[serde(flatten)]
    pub condition: Option<ConditionMessage>,
    pub interpolate: Option<InterpolationType>,
}
