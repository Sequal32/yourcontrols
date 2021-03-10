use msfs::legacy::execute_calculator_code;
use std::{borrow::Borrow, collections::HashMap, time::Instant};

use crate::interpolation::Interpolation;
use crate::util::{DatumKey, DatumValue, Time};

use super::{watcher::VariableWatcher, RcVariable, Syncable};

struct Datum {
    var: Option<RcVariable>,
    watch_data: Option<VariableWatcher>,
    // condition: Option<Condition>,
    interpolate: Option<Interpolation>,
    mapping: Option<Box<dyn Syncable>>,
}

impl Datum {
    fn get_changed_value(&mut self, tick: Time) -> Option<f64> {
        match self.watch_data.as_mut() {
            Some(w) => w.poll(tick),
            None => None,
        }
    }

    fn execute_mapping(&mut self, incoming_value: DatumValue) {
        match self.mapping.as_mut() {
            Some(m) => m.process_incoming(incoming_value),
            None => {}
        }
    }

    fn get_interpolation_calculator(&mut self, tick: Time) -> Option<String> {
        match self.interpolate.as_mut() {
            Some(i) => i.compute_interpolate_code(tick),
            None => None,
        }
    }

    fn queue_interpolate(&mut self, value: DatumValue, tick: Time) {
        if let Some(interpolate) = self.interpolate.as_mut() {
            interpolate.queue_data(value, tick);
        }
    }
}

struct ChangedDatum {
    pub key: DatumKey,
    pub value: DatumValue,
}

struct DeltaTimeChange {
    current_time: Time,
    instant: Instant,
}

impl DeltaTimeChange {
    pub fn new(start_time: Time) -> Self {
        Self {
            current_time: start_time,
            instant: Instant::now(),
        }
    }

    pub fn step(&mut self) -> Time {
        self.current_time += self.instant.elapsed().as_secs_f64();
        self.current_time
    }
}

struct DatumManager {
    datums: HashMap<u32, Datum>,

    interpolation_time: Option<DeltaTimeChange>,
    poll_time: DeltaTimeChange,
}

impl DatumManager {
    pub fn new(datums: HashMap<u32, Datum>) -> Self {
        Self {
            datums,
            interpolation_time: None,
            poll_time: DeltaTimeChange::new(0.0),
        }
    }

    fn execute_interpolate_strings(&self, interpolation_strings: Vec<String>) {
        execute_calculator_code::<f64>(&interpolation_strings.join(" "));
    }

    pub fn poll(&mut self) {
        let mut interpolation_strings = Vec::new();
        let mut changed_datums = Vec::new();

        // Execute stuff
        for (key, datum) in self.datums.iter_mut() {
            let interpolation_tick = self
                .interpolation_time
                .as_mut()
                .map(|x| x.step())
                .unwrap_or(0.0);

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

    pub fn process_incoming_data(&mut self, data: HashMap<DatumKey, DatumValue>, tick: Time) {
        // Set interpolation time
        if self.interpolation_time.is_none() {
            self.interpolation_time = Some(DeltaTimeChange::new(tick));
        }

        // Execute stuff
        for (key, value) in data {
            if let Some(datum) = self.datums.get_mut(&key) {
                datum.execute_mapping(value);
                datum.queue_interpolate(value, self.interpolation_time.as_mut().unwrap().step());
            }
        }
    }

    pub fn reset_interpolate(&mut self) {
        self.interpolation_time = None;
    }
}
