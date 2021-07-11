use crate::{DatumKey, DatumValue, InterpolationType, Time, VarId};
use rhai::Dynamic;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct EventMessage {
    pub name: String,
    pub param: Option<String>,
    #[serde(default)]
    pub param_reversed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptMessage {
    pub lines: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SettableMessage {
    Event(EventMessage),
    Var(VarType),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MappingArgsMessage {
    pub script_id: VarId,
    pub vars: Vec<VarId>,
    pub sets: Vec<VarId>,
    pub params: Vec<Dynamic>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MappingType<M> {
    Event,
    Var,
    Script(M),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
#[serde(untagged)]
pub enum VarTypeUntagged {
    WithUnits {
        name: String,
        units: String,
        index: Option<usize>,
    },
    Named {
        name: String,
    },
    Calculator {
        get: String,
        set: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub enum VarType {
    WithUnits {
        name: String,
        units: String,
        index: Option<usize>,
    },
    Named {
        name: String,
    },
    Calculator {
        get: String,
        set: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionMessage {
    pub script_id: VarId,
    pub vars: Vec<VarId>,
    pub params: Vec<Dynamic>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DatumMessage {
    pub var: Option<VarId>,
    pub watch_event: Option<String>,
    pub watch_period: Option<WatchPeriod>, // Watch variable
    pub conditions: Option<Vec<ConditionMessage>>,
    pub interpolate: Option<InterpolationType>,
    pub mapping: Option<MappingType<MappingArgsMessage>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangedDatum {
    pub key: DatumKey,
    pub value: DatumValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payloads {
    // Transmit to Sim
    SetMappings {
        datums: Vec<DatumMessage>,
        vars: Vec<VarType>,
        events: Vec<EventMessage>,
        scripts: Vec<ScriptMessage>,
    },

    WatchVariable {},
    WatchEvent {},
    MultiWatchVariable {},
    MultiWatchEvent {},
    ExecuteCalculator {},
    AddMapping {},
    SendIncomingValues {
        data: Vec<ChangedDatum>,
        time: Time,
    },
    RequestLvarNames,

    ResetInterpolation,
    Ping,
    ResetAll,
    // Receive from Sim
    LVars {
        data: Vec<String>,
    },
    VariableChange {
        changed: Vec<ChangedDatum>,
    },
    EventTriggered {},
    Pong,
}

/// Period where a variable becomes "Changed".
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WatchPeriod {
    Frame,
    Hz16,
    Second,
}

impl WatchPeriod {
    pub fn as_seconds_f64(&self) -> f64 {
        match self {
            WatchPeriod::Frame => 0.0,
            WatchPeriod::Hz16 => 0.26,
            WatchPeriod::Second => 1.0,
        }
    }

    pub fn default() -> Self {
        Self::Hz16
    }
}
