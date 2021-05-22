use yourcontrols_types::{DatumValue, Time, WatchPeriod};

use super::{RcVariable};

/// A pollable struct to watch for changes in variables in given intervals of `WatcherPeriod`.
pub struct VariableWatcher {
    var: RcVariable,
    last_period_tick: Option<Time>,
    last_value: DatumValue,
    did_change: bool,
    period: WatchPeriod,
}

impl VariableWatcher {
    pub fn new(var: RcVariable, period: WatchPeriod) -> Self {
        Self {
            var,
            last_period_tick: None,
            last_value: DatumValue::default(),
            did_change: false,
            period,
        }
    }

    fn can_process(&mut self, tick: Time) -> bool {
        let last_period_tick = match self.last_period_tick {
            Some(t) => t,
            None => return true, // Hasn't been initially updated yet
        };

        let last_update = tick - last_period_tick;
        return last_update >= self.period.as_seconds_f64();
    }

    /// Polls var for any changes in its given interval of `WatcherPeriod`.
    pub fn poll(&mut self, tick: Time) -> Option<DatumValue> {
        let current_value = self.var.get();
        let changed = current_value != self.last_value || self.did_change; // did_change used for if the period is not satisfied yet

        if !self.can_process(tick) || !changed {
            self.did_change = changed;
            return None;
        }

        self.did_change = false;
        self.last_value = current_value;
        self.last_period_tick = Some(tick);

        return Some(current_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MockVariable;
    use std::rc::Rc;

    #[test]
    fn test_frame() {
        let mut var = MockVariable::new();
        var.expect_get().times(2).return_const(100.0); // Init, Not changed
        var.expect_get().times(1).return_const(50.0); // Changed

        let mut watcher = VariableWatcher::new(Rc::new(var), WatchPeriod::Frame);

        // Init
        assert_eq!(watcher.poll(0.0), Some(100.0));
        // Not changed
        assert_eq!(watcher.poll(1.0), None);
        // Changed on every call
        assert_eq!(watcher.poll(1.0), Some(50.0));
    }

    fn test_period(period: WatchPeriod, unchanged_tick: Time, changed_tick: Time) {
        let mut var = MockVariable::new();
        var.expect_get().times(1).return_const(100.0); // Init
        var.expect_get().times(3).return_const(50.0); // Changed but under period, changed, not changed

        let mut watcher = VariableWatcher::new(Rc::new(var), period);

        // Init
        assert_eq!(watcher.poll(0.0), Some(100.0));
        // Changed but under period
        assert_eq!(watcher.poll(unchanged_tick), None);
        // Changed
        assert_eq!(watcher.poll(changed_tick), Some(50.0));
        // Not changed
        assert_eq!(watcher.poll(100.0), None);
    }

    #[test]
    fn test_hz() {
        test_period(WatchPeriod::Hz16, 0.15, 0.5);
    }

    #[test]
    fn test_second() {
        test_period(WatchPeriod::Second, 0.8, 1.0);
    }
}
