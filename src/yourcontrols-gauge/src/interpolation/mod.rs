use self::util::{interpolate_value, InterpolationType};
use crate::util::{DatumValue, Time};
use std::collections::{VecDeque};

mod util;

#[derive(Default, Clone)]
pub struct Packet {
    pub value: DatumValue,
    pub time: Time,
    pub current: DatumValue,
}

/// Handles interpolation of `Data` based on `InterpolationType`
pub struct Interpolation {
    newest_data_time: Time,
    did_init: bool,

    current_packet: Packet,
    interpolate_type: InterpolationType,
    calculator: String,
    queue: VecDeque<Packet>,
}

impl Interpolation {
    fn new(interpolate_type: InterpolationType, calculator: String) -> Self {
        Self {
            newest_data_time: Time::default(),
            did_init: false,

            queue: VecDeque::new(),
            current_packet: Packet::default(),
            interpolate_type,
            calculator,
        }
    }

    fn calculate_next_value(&mut self, tick: Time) {
        if self.queue.is_empty() {
            return;
        }

        for index in 0..self.queue.len() {
            let next_packet = self.queue.get(index).cloned().unwrap();

            // Packet is in the future
            if next_packet.time - tick > 0.0 {
                // How far we are into interpolating from the "current" packet to the next packet
                let mut alpha = (tick - self.current_packet.time)
                    / (next_packet.time - self.current_packet.time);

                // Haven't finished interpolating to the next packet yet
                if alpha <= 1.0 {
                    if index > 0 {
                        let new_current = self.queue.get(index - 1).cloned().unwrap();

                        for _ in 0..index {
                            self.queue.pop_front();
                        }

                        self.current_packet.time = new_current.time;
                        self.current_packet.value = new_current.value;

                        // Recalculate alpha between this
                        alpha = (tick - new_current.time) / (next_packet.time - new_current.time);
                    }
                } else {
                    continue;
                }

                self.current_packet.current = interpolate_value(
                    self.current_packet.value,
                    next_packet.value,
                    alpha,
                    &self.interpolate_type,
                );

                return;
            }
        }

        // No valid packets found, use very last value
        self.current_packet = self.queue.pop_back().unwrap();
    }

    pub fn queue_data(&mut self, data: DatumValue, time: Time) {
        let new_packet = Packet {
            value: data,
            time,
            current: data,
        };

        if self.did_init {
            self.queue.push_back(new_packet);
        } else {
            self.current_packet = new_packet;
            self.did_init = true;
        }

        self.newest_data_time = time;
    }

    pub fn compute_interpolate_code(&mut self, tick: Time) -> Option<String> {
        // No data queued yet
        if !self.did_init {
            return None;
        }
        // No data received for this key for a while, stop setting if not constant
        if tick - self.newest_data_time > 0.5 && !self.interpolate_type.is_constant() {
            return None;
        }

        self.calculate_next_value(tick);

        Some(format!(
            "{} (>{})",
            self.current_packet.current, self.calculator
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_sequence() {
        let mut interpolation = Interpolation::new(
            InterpolationType::Default,
            "A:PLANE ALTITUDE, Feet".to_string(),
        );
        interpolation.queue_data(0.0, 0.0);
        interpolation.queue_data(100.0, 1.0);
        interpolation.queue_data(200.0, 2.0);
        interpolation.queue_data(1000.0, 10.0);
        interpolation.queue_data(11000.0, 20.0);

        interpolation.calculate_next_value(0.0);
        assert_eq!(interpolation.current_packet.current, 0.0);
        // Next in sequence
        interpolation.calculate_next_value(0.5);
        assert_eq!(interpolation.current_packet.current, 50.0);
        interpolation.calculate_next_value(1.5);
        assert_eq!(interpolation.current_packet.current, 150.0);
        // Skip packets
        interpolation.calculate_next_value(15.5);
        assert_eq!(interpolation.current_packet.current, 6500.0);
        // Over time
        interpolation.calculate_next_value(50.0);
        assert_eq!(interpolation.current_packet.current, 11000.0);
    }

    #[test]
    fn test_empty_queue() {
        let mut interpolation = Interpolation::new(
            InterpolationType::Default,
            "A:PLANE ALTITUDE, Feet".to_string(),
        );

        assert!(interpolation.compute_interpolate_code(0.0).is_none())
    }

    #[test]
    fn test_correct_calculator() {
        let mut interpolation = Interpolation::new(
            InterpolationType::Default,
            "A:PLANE ALTITUDE, Feet".to_string(),
        );

        interpolation.queue_data(0.0, 0.0);
        interpolation.queue_data(100.0, 1.0);

        assert_eq!(
            interpolation
                .compute_interpolate_code(0.0)
                .expect("should exist"),
            "0 (>A:PLANE ALTITUDE, Feet)"
        )
    }
}
