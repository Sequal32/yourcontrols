use self::util::{get_time_as_seconds, Data, InterpolationType, Packet};
use crate::util::{DatumKey, DatumValue, Time};
use std::collections::HashMap;

mod util;

/// Handles interpolation of `Data` based on `InterpolationType`
pub struct Interpolation {
    data: HashMap<DatumKey, Data>,
    last_called: Time,
    current_time: Time,
    newest_data_time: Time,
}

impl Interpolation {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            last_called: 0.0,
            current_time: 0.0,
            newest_data_time: 0.0,
        }
    }

    /// Queues an f64 to be interpolated to `value` at time `time`.
    fn queue_value(&mut self, id: u32, value: f64, time: f64) {
        let mut data = self.data.get_mut(&id).unwrap();

        let new_packet = Packet {
            value,
            time,
            current: value,
        };

        if data.did_init {
            data.queue.push_back(new_packet)
        } else {
            data.current_packet = new_packet;
            data.did_init = true;
        }
    }

    pub fn queue_data(&mut self, data: &HashMap<DatumKey, DatumValue>, time: f64) {
        for (id, value) in data {
            self.queue_value(*id, *value, time)
        }

        // Allow interpolate to run
        if self.newest_data_time == 0.0 {
            self.current_time = time - 0.1
        }

        self.newest_data_time = time;
    }

    /// Maps some options to `id`.
    ///
    /// `calculator` is a String to be executed using `execute_calculator_code` in the format of Type:Name,Units or just Type:Name.
    /// Examples of `calculator`: `A:PLANE LATITUDE, Dergrees`, `K:AXIS_ELEVATOR_SET`
    pub fn set_data_options(
        &mut self,
        id: u32,
        calculator: String,
        interpolate_type: InterpolationType,
    ) {
        self.data
            .insert(id, Data::new(calculator, interpolate_type));
    }

    fn interpolate_code_for_time(&mut self, time: Time) -> Option<String> {
        let mut calculator = String::new();

        for (_, data) in self.data.iter_mut() {
            if !data.did_init {
                continue;
            }
            // No data received for this key for a while, stop setting if not constant
            if self.newest_data_time - data.last_received_time > 0.5
                && !data.interpolate_type.is_constant()
            {
                continue;
            }

            data.calculate_next_value(time);

            calculator += &format!("{} (>{})", data.current_packet.current, data.calculator);
            calculator += " ";
        }

        Some(calculator)
    }

    fn compute_interpolate_code_from_seconds(&mut self, current_time: Time) -> Option<String> {
        let mut delta_time = current_time - self.last_called;

        self.last_called = current_time;

        // No data queued yet
        if self.current_time == 0.0 {
            return None;
        }

        // Difference between last data received and the current time. Need to correct for large errors.
        let diff = self.newest_data_time - self.current_time;

        // Catch up
        if diff > 0.2 {
            delta_time += diff - 0.2;
        // No data received for a while
        } else if diff < -0.3 {
            self.reset();
            return None;
        }

        self.current_time += delta_time;

        self.interpolate_code_for_time(current_time)
    }

    /// Returns a string that should be executed using `execute_calculator_code` with interpolated values.
    pub fn compute_interpolate_code(&mut self) -> Option<String> {
        self.compute_interpolate_code_from_seconds(get_time_as_seconds())
    }

    /// Resets all mapped data.
    pub fn reset(&mut self) {
        self.data.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_queue_data() {
        let mut interpolation = Interpolation::new();
        interpolation.set_data_options(
            0,
            "A:PLANE LATITUDE, Degrees".to_string(),
            InterpolationType::DefaultConstant,
        );

        // Basic initial value

        let mut new_data = HashMap::new();
        new_data.insert(0, 50.0);
        interpolation.queue_data(&new_data, 0.0);

        assert!(interpolation
            .compute_interpolate_code_from_seconds(0.0)
            .expect("should be some")
            .contains("50 (>A:PLANE LATITUDE, Degrees)"),);

        // Should support multiple
        interpolation.set_data_options(
            1,
            "A:PLANE ALTITUDE, Feet".to_string(),
            InterpolationType::DefaultConstant,
        );

        new_data.insert(0, 200.0);
        new_data.insert(1, 200.0);

        interpolation.queue_data(&new_data, 1.0);

        let calculator = interpolation
            .compute_interpolate_code_from_seconds(0.5)
            .expect("should be some");

        assert!(calculator.contains("125 (>A:PLANE LATITUDE, Degrees)"));
        assert!(calculator.contains("200 (>A:PLANE ALTITUDE, Feet)"));
    }
}
