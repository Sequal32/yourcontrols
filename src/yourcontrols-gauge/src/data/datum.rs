#[cfg(any(target_arch = "wasm32"))]
use msfs::legacy::execute_calculator_code;
use msfs::sim_connect::SimConnectRecv;
use std::{collections::HashMap, time::Instant};

use crate::util::{DatumKey, DatumValue, Time};
use crate::{interpolation::Interpolation, sync::Condition};

use super::util::{ChangedDatum, DeltaTimeChange};
use super::watcher::VariableWatcher;
use super::KeyEvent;
use super::{RcVariable, Syncable};

/// A Datum can watch for changes in variables, conditionally execute a mapping (event/setting the variable) or be interpolated to a value every frame.
pub struct Datum {
    var: Option<RcVariable>,
    watch_event: Option<KeyEvent>,
    watch_data: Option<VariableWatcher>,
    condition: Option<Condition>,
    interpolate: Option<Interpolation>,
    mapping: Option<Box<dyn Syncable>>,
}

impl Datum {
    fn get_changed_value(&mut self, tick: Time) -> Option<DatumValue> {
        self.watch_data.as_mut()?.poll(tick)
    }

    fn get_interpolation_calculator(&mut self, tick: Time) -> Option<String> {
        self.interpolate.as_mut()?.compute_interpolate_code(tick)
    }

    fn queue_interpolate(&mut self, value: DatumValue, tick: Time) -> Option<()> {
        self.interpolate.as_mut()?.queue_data(value, tick);
        Some(())
    }

    fn process_sim_event_id(&mut self, id: u32) -> Option<()> {
        let event = self.watch_event.as_mut()?;
        if id == event.id {
            event.increment_count()
        }
        Some(())
    }

    fn execute_mapping(&mut self, incoming_value: DatumValue) -> Option<()> {
        if self.is_condition_satisifed(incoming_value) {
            self.mapping.as_mut()?.process_incoming(incoming_value);
        }
        Some(())
    }

    fn is_condition_satisifed(&self, incoming_value: DatumValue) -> bool {
        self.condition
            .as_ref()
            .map_or(true, |x| x.is_satisfied(incoming_value))
    }
}

pub struct DatumManager {
    datums: HashMap<u32, Datum>,
    interpolation_time: Option<DeltaTimeChange>,
    poll_time: DeltaTimeChange,
}

impl DatumManager {
    pub fn new() -> Self {
        Self {
            datums: HashMap::new(),
            interpolation_time: None,
            poll_time: DeltaTimeChange::new(0.0),
        }
    }

    #[cfg(any(target_arch = "wasm32"))]
    fn execute_interpolate_strings(&self, interpolation_strings: Vec<String>) {
        execute_calculator_code::<()>(&interpolation_strings.join(" "));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_interpolate_strings(&self, interpolation_strings: Vec<String>) {}

    fn get_interpolation_tick(&mut self) -> Time {
        self.interpolation_time
            .as_mut()
            .map(|x| x.step())
            .unwrap_or(0.0)
    }

    /// Adds a datum.
    pub fn add_datum(&mut self, key: DatumKey, datum: Datum) {
        self.datums.insert(key, datum);
    }

    /// Runs interpolation and watcher tasks for each datum.
    pub fn poll(&mut self) {
        let mut interpolation_strings = Vec::new();
        let mut changed_datums = Vec::new();

        let interpolation_tick = self.get_interpolation_tick();
        // Execute stuff
        for (key, datum) in self.datums.iter_mut() {
            if let Some(interpolate_string) = datum.get_interpolation_calculator(interpolation_tick)
            {
                interpolation_strings.push(interpolate_string);
            }

            if let Some(value) = datum.get_changed_value(self.poll_time.step()) {
                changed_datums.push(ChangedDatum { key: *key, value })
            }
        }

        self.execute_interpolate_strings(interpolation_strings);
    }

    /// Incoming data is queued for interpolation or is used to execute datum mappings.
    pub fn process_incoming_data(&mut self, data: HashMap<DatumKey, DatumValue>, tick: Time) {
        // Set interpolation time
        if self.interpolation_time.is_none() {
            self.interpolation_time = Some(DeltaTimeChange::new(tick - 0.05));
        }

        // Execute stuff
        for (key, value) in data {
            if let Some(datum) = self.datums.get_mut(&key) {
                datum.execute_mapping(value);
                datum.queue_interpolate(value, tick);
            }
        }
    }

    /// Stops and resets the timing for interpolation.
    pub fn reset_interpolate(&mut self) {
        self.interpolation_time = None;
    }

    pub fn process_sim_event_id(&mut self, id: u32) {
        for (key, value) in self.datums.iter_mut() {
            value.process_sim_event_id(id);
        }
    }
}
