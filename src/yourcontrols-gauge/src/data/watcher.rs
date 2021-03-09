use std::collections::HashMap;

use super::{diff::DiffChecker, RcVariable, Variable};
use crate::util::{DatumKey, DatumValue};

const HZ16_MS: u64 = 375;
const SECOND_MS: u64 = 1000;

/// Period where a variable becomes "Changed".
///
/// Frame: 1000ms
/// Hz16: 375ms
pub enum WatchPeriod {
    Frame,
    Hz16,
    Second,
}

/// Contains the information of a changed variable.
pub struct ChangedVar {
    pub key: DatumKey,
    pub value: DatumValue,
}

struct WatchData {
    variable: RcVariable,
    period: WatchPeriod,
    did_change: bool,
    did_init_send: bool,
}

impl WatchData {
    fn new(variable: RcVariable, period: WatchPeriod) -> Self {
        Self {
            variable,
            period,
            did_change: false,
            did_init_send: false,
        }
    }
}

fn can_process(last_tick: &mut u64, current_tick: u64, period_ms: u64) -> bool {
    if current_tick - *last_tick >= period_ms {
        *last_tick = current_tick;
        return true;
    }
    return false;
}

/// A pollable struct to watch for changes in variables in given intervals of `WatcherPeriod`.
pub struct VariableWatcher {
    vars: HashMap<DatumKey, WatchData>,
    diff_checker: DiffChecker<DatumKey, DatumValue>,

    tick: u64,
    last_hz16: u64,
    last_second: u64,
}

impl VariableWatcher {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            diff_checker: DiffChecker::new(),
            tick: 0,
            last_hz16: 0,
            last_second: 0,
        }
    }

    /// Adds a var to be watched.
    pub fn add_var(&mut self, datum_key: DatumKey, variable: RcVariable, period: WatchPeriod) {
        self.vars
            .insert(datum_key, WatchData::new(variable, period));
    }

    /// Polls all watching vars for any changes in their given intervals of `WatcherPeriod`.
    pub fn poll(&mut self, delta_time_ms: u64) -> Vec<ChangedVar> {
        let mut changed_vars = Vec::new();

        self.tick += delta_time_ms;

        for (key, watch_data) in self.vars.iter_mut() {
            let current_value = watch_data.variable.borrow().get();
            let changed = self.diff_checker.add(*key, current_value) || watch_data.did_change; // did_change used for if the period is not satisfied yet

            let can_process = match watch_data.period {
                WatchPeriod::Frame => true,
                WatchPeriod::Hz16 => can_process(&mut self.last_hz16, self.tick, HZ16_MS),
                WatchPeriod::Second => can_process(&mut self.last_second, self.tick, SECOND_MS),
            };

            if (!can_process || !changed) && watch_data.did_init_send {
                watch_data.did_change = changed;
                continue;
            }

            watch_data.did_change = false;
            watch_data.did_init_send = true;

            changed_vars.push(ChangedVar {
                key: *key,
                value: current_value,
            });
        }

        changed_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::test::{get_test_variable, TestVariable};

    #[test]
    fn test_var_watcher_frame() {
        let var = get_test_variable(100.0);
        let mut watcher = VariableWatcher::new();
        watcher.add_var(0, var.clone(), WatchPeriod::Frame);
        // Should get the variable when first added no matter the period
        let vars = watcher.poll(0);
        assert_eq!(vars[0].value, 100.0);
        assert_eq!(vars[0].key, 0);

        // Unchanged
        assert_eq!(watcher.poll(1).len(), 0);

        // Changed
        var.borrow_mut().set_new_value(1.0);

        let vars = watcher.poll(1);
        assert_eq!(vars[0].value, 1.0);
        assert_eq!(vars[0].key, 0);
    }

    macro_rules! test_period {
        ($period: ident, $poll_unchanged: expr, $poll_changed: expr) => {
            let var = get_test_variable(100.0);
            let mut watcher = VariableWatcher::new();
            watcher.add_var(0, var.clone(), Period::$period);
            // Should get the variable when first added no matter the period
            let vars = watcher.poll(0);
            assert_eq!(vars[0].value, 100.0);
            assert_eq!(vars[0].key, 0);

            // Changed
            var.borrow_mut().set_new_value(1.0);
            // Period not reached
            assert_eq!(watcher.poll($poll_unchanged).len(), 0);

            // Period reached

            let vars = watcher.poll($poll_changed);
            assert_eq!(vars[0].value, 1.0);
            assert_eq!(vars[0].key, 0);
        };
    }

    #[test]
    fn test_var_watcher_second() {
        test_period!(Second, 500, SECOND_MS);
    }

    #[test]
    fn test_var_watcher_hz() {
        test_period!(Hz16, 16, HZ16_MS);
    }
}
