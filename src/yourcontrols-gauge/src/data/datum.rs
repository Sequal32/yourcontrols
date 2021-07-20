#[cfg(any(target_arch = "wasm32"))]
use msfs::legacy::execute_calculator_code;
use rhai::Dynamic;

use std::collections::HashMap;
use yourcontrols_types::{
    ChangedDatum, DatumKey, DatumValue, InterpolationType, MappingType, Time, VarId,
};

use crate::interpolation::Interpolation;
use crate::sync::SCRIPTING_ENGINE;

use super::watcher::VariableWatcher;
use super::KeyEvent;
use super::{util::DeltaTimeChange, RcSettable};
use super::{RcVariable, Syncable};

const ENSURE_VALUE_IS_SET_POLL_TIME: f64 = 2.0;

#[cfg_attr(test, mockall::automock)]
pub trait DatumTrait {
    fn get_changed_value(&mut self, tick: Time) -> Option<DatumValue>;
    fn interpolate(&mut self, tick: Time) -> Option<()>;
    fn queue_interpolate(&mut self, value: DatumValue, tick: Time) -> Option<()>;
    fn process_sim_event_id(&mut self, id: u32) -> Option<()>;
    fn execute_mapping(&mut self, incoming_value: DatumValue) -> Option<()>;
    fn execute_mapping_timed(&mut self, incoming_value: DatumValue) -> Option<()>;
    fn poll_mapping(&mut self) -> Option<()>;
    fn are_conditions_satisfied(&self, incoming_value: DatumValue) -> bool;
    fn has_mapping(&self) -> bool;
    fn is_interpolated(&self) -> bool;
}
/// A Datum can watch for changes in variables, conditionally execute a mapping (event/setting the variable) or be interpolated to a value every frame.
#[derive(Default)]
pub struct Datum {
    pub var: Option<RcVariable>,
    pub watch_event: Option<KeyEvent>,
    pub watch_data: Option<VariableWatcher>,
    pub conditions: Option<Vec<Condition>>,
    pub interpolate: Option<Interpolation>,
    pub mapping: Option<MappingType<MappingArgs>>,
    execute_loop_time: Option<DeltaTimeChange>,
    last_incoming_value: Option<DatumValue>,
}

impl Datum {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_var(&mut self, var: RcVariable) {
        self.var = Some(var);
    }

    pub fn with_watch_event(&mut self, watch_event: KeyEvent) {
        self.watch_event = Some(watch_event);
    }

    pub fn with_watch_data(&mut self, watch_data: VariableWatcher) {
        self.watch_data = Some(watch_data);
    }

    pub fn with_conditions(&mut self, conditions: Vec<Condition>) {
        self.conditions = Some(conditions);
    }

    pub fn with_interpolate(&mut self, interpolate_type: InterpolationType) {
        self.interpolate = Some(Interpolation::new(interpolate_type));
    }

    pub fn with_mapping(&mut self, mapping: MappingType<MappingArgs>) {
        self.mapping = Some(mapping);
    }
}

impl DatumTrait for Datum {
    fn get_changed_value(&mut self, tick: Time) -> Option<DatumValue> {
        self.watch_data.as_mut()?.poll(tick)
    }

    fn interpolate(&mut self, tick: Time) -> Option<()> {
        let new_value = self.interpolate.as_mut()?.get_value_at(tick)?;
        self.execute_mapping(new_value)
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
        if !self.are_conditions_satisfied(incoming_value) {
            return None;
        };

        self.last_incoming_value = Some(incoming_value);

        match self.mapping.as_mut()? {
            MappingType::Event => self.watch_event.as_ref()?.process_incoming(incoming_value),
            MappingType::Var => self.var.as_ref()?.set(incoming_value),
            MappingType::Script(args) => SCRIPTING_ENGINE.with(|x| {
                println!(
                    "{:?}",
                    x.borrow().process_incoming_value(
                        args.script_id,
                        incoming_value,
                        args.vars.clone(),
                        args.sets.clone(),
                        args.params.clone(),
                    )
                );
            }),
        }

        Some(())
    }

    /// Executes the mapping and begins the 2 second timer to ensure the value is set
    fn execute_mapping_timed(&mut self, incoming_value: DatumValue) -> Option<()> {
        self.execute_loop_time = Some(DeltaTimeChange::new(0.0));
        self.execute_mapping(incoming_value)
    }

    /// Determines if the newest value was set within the last 2 seconds, and if so, re-executes the mapping
    fn poll_mapping(&mut self) -> Option<()> {
        let should_try_set = self
            .execute_loop_time
            .as_mut()
            .map(|d| d.step() <= ENSURE_VALUE_IS_SET_POLL_TIME)
            .unwrap_or(false);

        if should_try_set {
            self.execute_mapping(self.last_incoming_value?)
        } else {
            None
        }
    }

    fn are_conditions_satisfied(&self, incoming_value: DatumValue) -> bool {
        let conditions = match self.conditions.as_ref() {
            Some(c) => c,
            None => return true,
        };

        let mut satisfied = false;

        for condition in conditions {
            SCRIPTING_ENGINE.with(|x| {
                satisfied &= x
                    .borrow()
                    .evaluate_condition(
                        condition.script_id,
                        incoming_value,
                        condition.vars.clone(),
                        condition.params.clone(),
                    )
                    .unwrap_or(false);
            });
        }

        return satisfied;
    }

    fn has_mapping(&self) -> bool {
        self.mapping.is_some()
    }

    fn is_interpolated(&self) -> bool {
        self.interpolate.is_some()
    }
}

pub struct DatumManager<T> {
    datums: HashMap<DatumKey, T>,
    interpolation_time: Option<DeltaTimeChange>,
    poll_time: DeltaTimeChange,
}

impl<T: DatumTrait> DatumManager<T> {
    pub fn new() -> Self {
        Self {
            datums: HashMap::new(),
            interpolation_time: None,
            poll_time: DeltaTimeChange::new(0.0),
        }
    }

    fn get_interpolation_tick(&mut self) -> Time {
        self.interpolation_time
            .as_mut()
            .map(|x| x.step())
            .unwrap_or(0.0)
    }

    /// Adds a datum.
    pub fn add_datum(&mut self, key: DatumKey, datum: T) {
        self.datums.insert(key, datum);
    }

    /// Runs interpolation and watcher tasks for each datum.
    pub fn poll(&mut self) -> Vec<ChangedDatum> {
        let mut changed_datums = Vec::new();

        let interpolation_tick = self.get_interpolation_tick();
        // Execute stuff
        for (key, datum) in self.datums.iter_mut() {
            datum.interpolate(interpolation_tick);

            if let Some(value) = datum.get_changed_value(self.poll_time.step()) {
                changed_datums.push(ChangedDatum { key: *key, value })
            }
        }

        changed_datums
    }

    /// Incoming data is queued for interpolation or is used to execute datum mappings.
    pub fn process_incoming_data(&mut self, data: Vec<ChangedDatum>, interpolate_tick: Time) {
        // Set interpolation time
        if self.interpolation_time.is_none() {
            self.interpolation_time = Some(DeltaTimeChange::new(interpolate_tick - 0.05));
        }

        // Execute stuff
        for new_datum in data {
            let datum = match self.datums.get_mut(&new_datum.key) {
                Some(d) => d,
                None => continue,
            };

            if datum.has_mapping() {
                datum.execute_mapping(new_datum.value);
            } else if datum.is_interpolated() {
                datum.queue_interpolate(new_datum.value, interpolate_tick);
            }
        }
    }

    /// Stops and resets the timing for interpolation.
    pub fn reset_interpolate_time(&mut self) {
        self.interpolation_time = None;
    }

    pub fn process_sim_event_id(&mut self, id: u32) {
        for (_key, value) in self.datums.iter_mut() {
            value.process_sim_event_id(id);
        }
    }
}

#[cfg_attr(not(test), derive(Debug))]
pub struct MappingArgs {
    pub script_id: VarId,
    pub vars: Vec<RcVariable>,
    pub sets: Vec<RcSettable>,
    pub params: Vec<Dynamic>,
}

pub struct Condition {
    pub script_id: VarId,
    pub vars: Vec<RcVariable>,
    pub params: Vec<Dynamic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_incoming(value: DatumValue) -> Vec<ChangedDatum> {
        let mut datums = Vec::new();
        datums.push(ChangedDatum {
            key: 0,
            value: value,
        });
        datums
    }

    #[test]
    fn test_empty_datum() {
        let mut datum = Datum::default();

        assert_eq!(datum.get_changed_value(0.0), None);
        assert_eq!(datum.interpolate(0.0), None);
        assert_eq!(datum.queue_interpolate(0.0, 0.0), None);
        assert_eq!(datum.process_sim_event_id(0), None);
        assert_eq!(datum.execute_mapping(0.0), None);
        assert_eq!(datum.are_conditions_satisfied(0.0), true);
        assert_eq!(datum.has_mapping(), false);
        assert_eq!(datum.is_interpolated(), false);
    }

    #[test]
    fn test_incoming_datum() {
        let mut mock = MockDatumTrait::new();
        mock.expect_has_mapping().once().return_const(true);
        mock.expect_execute_mapping().once().return_const(None);

        let mut manager = DatumManager::<MockDatumTrait>::new();
        manager.add_datum(0, mock);
        manager.process_incoming_data(get_test_incoming(0.0), 0.0);
    }

    #[test]
    fn test_poll_calls() {
        let mut mock = MockDatumTrait::new();
        mock.expect_get_changed_value().once().return_const(1.0);
        mock.expect_interpolate().once().return_const(None);

        let mut manager = DatumManager::<MockDatumTrait>::new();
        manager.add_datum(0, mock);

        let changed = manager.poll();

        assert_eq!(changed.get(0).expect("should've returned").value, 1.0);
    }
}
