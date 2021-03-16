#[cfg(any(target_arch = "wasm32"))]
use msfs::legacy::execute_calculator_code;

use std::collections::HashMap;
use yourcontrols_types::{DatumKey, DatumValue, SyncPermission, SyncPermissionState, Time};

use crate::{interpolation::Interpolation, sync::Condition};

use super::util::{ChangedDatum, DeltaTimeChange};
use super::watcher::VariableWatcher;
use super::KeyEvent;
use super::{RcVariable, Syncable};

#[cfg_attr(test, mockall::automock)]
pub trait DatumTrait {
    fn get_changed_value(&mut self, tick: Time) -> Option<DatumValue>;
    fn get_interpolation_calculator(&mut self, tick: Time) -> Option<String>;
    fn queue_interpolate(&mut self, value: DatumValue, tick: Time) -> Option<()>;
    fn process_sim_event_id(&mut self, id: u32) -> Option<()>;
    fn execute_mapping(&mut self, incoming_value: DatumValue) -> Option<()>;
    fn is_condition_satisifed(&self, incoming_value: DatumValue) -> bool;
    fn can_execute(&self, sync_permission: &SyncPermissionState) -> bool;
}
/// A Datum can watch for changes in variables, conditionally execute a mapping (event/setting the variable) or be interpolated to a value every frame.
#[derive(Default)]
pub struct Datum {
    pub var: Option<RcVariable>,
    pub watch_event: Option<KeyEvent>,
    pub watch_data: Option<VariableWatcher>,
    pub condition: Option<Condition>,
    pub interpolate: Option<Interpolation>,
    pub mapping: Option<Box<dyn Syncable>>,
    pub sync_permission: Option<SyncPermission>,
}

impl DatumTrait for Datum {
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

    fn can_execute(&self, sync_permission: &SyncPermissionState) -> bool {
        self.sync_permission
            .as_ref()
            .map(|x| match x {
                SyncPermission::Shared => true,
                SyncPermission::Master => sync_permission.master,
                SyncPermission::Server => sync_permission.server,
                SyncPermission::Init => sync_permission.init,
            })
            .unwrap_or(true)
    }
}

pub struct DatumManager {
    datums: HashMap<u32, Box<dyn DatumTrait>>,
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
    fn execute_interpolate_strings(&self, _interpolation_strings: Vec<String>) {}

    fn get_interpolation_tick(&mut self) -> Time {
        self.interpolation_time
            .as_mut()
            .map(|x| x.step())
            .unwrap_or(0.0)
    }

    /// Adds a datum.
    pub fn add_datum(&mut self, key: DatumKey, datum: Box<dyn DatumTrait>) {
        self.datums.insert(key, datum);
    }

    /// Runs interpolation and watcher tasks for each datum.
    pub fn poll(&mut self, sync_permission: &SyncPermissionState) -> Vec<ChangedDatum> {
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
                // Based on sync permission
                if !datum.can_execute(sync_permission) {
                    continue;
                }
                changed_datums.push(ChangedDatum { key: *key, value })
            }
        }

        self.execute_interpolate_strings(interpolation_strings);

        changed_datums
    }

    /// Incoming data is queued for interpolation or is used to execute datum mappings.
    pub fn process_incoming_data(
        &mut self,
        data: HashMap<DatumKey, DatumValue>,
        tick: Time,
        sync_permission: &SyncPermissionState,
    ) {
        // Set interpolation time
        if self.interpolation_time.is_none() {
            self.interpolation_time = Some(DeltaTimeChange::new(tick - 0.05));
        }

        // Execute stuff
        for (key, value) in data {
            if let Some(datum) = self.datums.get_mut(&key) {
                if !datum.can_execute(sync_permission) {
                    return;
                }

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
        for (_key, value) in self.datums.iter_mut() {
            value.process_sim_event_id(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_incoming(value: DatumValue) -> HashMap<DatumKey, DatumValue> {
        let mut datums = HashMap::new();
        datums.insert(0, value);
        datums
    }

    #[test]
    fn test_empty_datum() {
        let mut datum = Datum::default();

        assert_eq!(datum.get_changed_value(0.0), None);
        assert_eq!(datum.get_interpolation_calculator(0.0), None);
        assert_eq!(datum.queue_interpolate(0.0, 0.0), None);
        assert_eq!(datum.process_sim_event_id(0), None);
        assert_eq!(datum.execute_mapping(0.0), None);
        assert_eq!(datum.is_condition_satisifed(0.0), true);
        assert_eq!(datum.can_execute(&SyncPermissionState::default()), true);
    }

    #[test]
    fn test_incoming_datum() {
        let mut mock = MockDatumTrait::new();
        mock.expect_can_execute().once().return_const(true);
        mock.expect_queue_interpolate().once().return_const(None);
        mock.expect_execute_mapping().once().return_const(None);

        let mut manager = DatumManager::new();
        manager.add_datum(0, Box::new(mock));
        manager.process_incoming_data(get_test_incoming(0.0), 0.0, &SyncPermissionState::default());
    }

    #[test]
    fn test_poll_calls() {
        let mut mock = MockDatumTrait::new();
        mock.expect_can_execute().once().return_const(true);
        mock.expect_get_interpolation_calculator()
            .once()
            .return_const("Test".to_string());
        mock.expect_get_changed_value().once().return_const(1.0);

        let mut manager = DatumManager::new();
        manager.add_datum(0, Box::new(mock));

        let changed = manager.poll(&SyncPermissionState::default());

        assert_eq!(changed.get(0).expect("should've returned").value, 1.0);
    }
}
