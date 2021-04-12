use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct SyncPermissionState {
    pub server: bool,
    pub master: bool,
    pub init: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SyncPermission {
    Shared,
    Master,
    Server,
    Init,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionMessage {
    pub script_name: String,
    pub options: Vec<Dynamic>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InterpolateMessage {
    pub calculator: String,
    pub interpolate_type: InterpolationType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DatumMessage {
    pub var: Option<VarId>,
    pub watch_event: Option<String>,
    pub watch_period: Option<WatchPeriod>, // Watch variable
    pub condition: Option<ConditionMessage>,
    pub interpolate: Option<InterpolateMessage>,
    pub mapping: Option<MappingType<MappingArgsMessage>>,
    pub sync_permission: Option<SyncPermission>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangedDatum {
    pub key: DatumKey,
    pub value: DatumValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payloads {
    // Transmit to Sim
    SetDatums {
        datums: Vec<DatumMessage>,
    },
    SetVars {
        vars: Vec<VarType>,
    },
    SetEvents {
        events: Vec<EventMessage>,
    },
    SetScripts {
        scripts: Vec<ScriptMessage>,
    },

    WatchVariable {},
    WatchEvent {},
    MultiWatchVariable {},
    MultiWatchEvent {},
    ExecuteCalculator {},
    AddMapping {},
    SendIncomingValues {
        data: HashMap<DatumKey, DatumValue>,
        time: Time,
    },
    UpdateSyncPermission {
        new: SyncPermissionState,
    },

    ResetInterpolation,
    Ping,
    ResetAll,
    // Receive from Sim
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
}
