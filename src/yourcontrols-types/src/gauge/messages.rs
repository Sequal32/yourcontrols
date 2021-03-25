use std::collections::HashMap;

use crate::{DatumKey, DatumValue, InterpolationType, Time, VarId};
use rhai::Dynamic;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum MappingType<P>
where
    P: Eq,
{
    ToggleSwitch {
        event_name: String,
        off_event_name: Option<String>,
        #[serde(default)]
        switch_on: bool,
        event_param: Option<P>,
    },
    NumSet {
        event_name: String,
        swap_event_name: Option<String>,
        multiply_by: Option<DatumValue>,
        add_by: Option<DatumValue>,
        event_param: Option<P>,
    },
    NumIncrement {
        up_event_name: String,
        down_event_name: String,
        increment_amount: DatumValue,
        #[serde(default)]
        pass_difference: bool,
    },
    NumDigitSet {
        inc_events: Vec<String>,
        dec_events: Vec<String>,
    },
    Event,
    Var,
    Script(M),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    pub mapping: Option<MappingType<u32>>,
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
