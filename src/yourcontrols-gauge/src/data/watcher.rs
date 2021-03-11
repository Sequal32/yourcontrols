use yourcontrols_types::{DatumValue, Time};

use super::{RcVariable, Variable};

/// Period where a variable becomes "Changed".
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
        let current_value = self.var.borrow().get();
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

    use crate::util::test::get_test_variable;

    #[test]
    fn test_frame() {
        let var = get_test_variable(100.0);
        let mut watcher = VariableWatcher::new(var.clone(), WatchPeriod::Frame);

        // Init
        assert_eq!(watcher.poll(0.0), Some(100.0));
        // Not changed
        assert_eq!(watcher.poll(1.0), None);
        // Changed on every call
        var.borrow_mut().set_new_value(50.0);
        assert_eq!(watcher.poll(1.0), Some(50.0));
    }

    fn test_period(period: WatchPeriod, unchanged_tick: Time, changed_tick: Time) {
        let var = get_test_variable(100.0);
        let mut watcher = VariableWatcher::new(var.clone(), period);

        // Init
        assert_eq!(watcher.poll(0.0), Some(100.0));
        // Not changed
        assert_eq!(watcher.poll(50.0), None);
        // Changed but under period
        var.borrow_mut().set_new_value(50.0);
        assert_eq!(watcher.poll(unchanged_tick), None);
        // Changed
        assert_eq!(watcher.poll(changed_tick), Some(50.0));
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
