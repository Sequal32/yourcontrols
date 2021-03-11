use crate::{DatumValue, InterpolationType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MappingType {
    ToggleSwitch {
        event_name: String,
        off_event_name: Option<String>,
        #[serde(default)]
        switch_on: bool,
    },
    NumSet {
        event_name: String,
        swap_event_name: Option<String>,
        multiply_by: Option<DatumValue>,
        add_by: Option<DatumValue>,
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
    Var,
    ProgramAction, // TODO:
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConditionMessage {
    equals: Option<DatumValue>,
    less_than: Option<DatumValue>,
    greater_than: Option<DatumValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterpolateMessage {
    calculator: String,
    interpolate_type: InterpolationType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatumMessage {
    var: Option<VarType>,
    watch_event: Option<String>, // Event name,
    should_watch: bool,          // Watch variable
    condition: Option<ConditionMessage>,
    interpolate: Option<InterpolateMessage>,
    mapping: Option<MappingType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Payloads {
    // Transmit to Sim
    SetDatums { datums: Vec<DatumMessage> },

    WatchVariable {},
    WatchEvent {},
    MultiWatchVariable {},
    MultiWatchEvent {},
    ExecuteCalculator {},
    AddMapping {},
    SendIncomingValues {},

    QueueInterpolationData {},
    SetInterpolationData {},
    StopInterpolation {},
    ResetInterpolation {},

    Ping,
    ResetAll,
    // Receive from Sim
    VariableChange {},
    EventTriggered {},
    Pong,
}
