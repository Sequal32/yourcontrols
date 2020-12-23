use std::{collections::{HashMap}, time::Instant};
use serde::Deserialize;

use crate::{util::VarReaderTypes, varreader::SimValue};

const DELAY_TIME: f64 = 0.1;

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct InterpolateOptions {
    overshoot: f64, // How many seconds to interpolate for after interpolation_time has been reached
    wrap360: bool,
    wrap180: bool,
    wrap90: bool
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            overshoot: 0.0,
            wrap360: false,
            wrap180: false,
            wrap90: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Packet {
    time: f64,
    value: f64
}

fn interpolate_value(from: f64, to: f64, alpha: f64, options: Option<&InterpolateOptions>) -> f64 {
    if let Some(options) = options {
        if options.wrap360 {
            return interpolate_f64_degrees(from, to, alpha);
        } else if options.wrap180 {
            return interpolate_f64_degrees_180(from, to, alpha);
        } else if options.wrap90 {
            return interpolate_f64_degrees_90(from, to, alpha);
        } else {
            return interpolate_f64(from, to, alpha);
        }
    } else {
        return interpolate_f64(from, to, alpha);
    }
}

// Returns if set
fn set_next_value(current: &mut Packet, current_time: f64, queue: &mut Vec<Packet>, options: Option<&InterpolateOptions>) -> Option<f64> {
    if queue.len() == 0 {return None;}

    for i in 0..queue.len() {
        let packet = queue.get(i).unwrap();
        

        if packet.time-current_time > 0.0 {
            let mut alpha = (current_time-current.time)/(packet.time-current.time);

            if alpha <= 1.0 {
                let to_value = packet.value;
                let time = packet.time;

                if i > 0 {
                    // Set time to previous packet to interpolate from
                    *current = queue.get(i-1).unwrap().clone();
                    // Remove all packets before index
                    queue.drain(0..i);
                    alpha = (current_time-current.time)/(time-current.time);
                }

                return Some(interpolate_value(current.value, to_value, alpha, options));
            }
        }

    }

    None
}

pub struct Interpolate {
    current_data: HashMap<String, Packet>,
    data_queue: HashMap<String, Vec<Packet>>,
    options: HashMap<String, InterpolateOptions>,
    step_last_called: Instant,

    current_time: f64,
    newest_data_time: f64
}

impl Interpolate {
    pub fn new() -> Self {
        Self {
            current_data: HashMap::new(),
            data_queue: HashMap::new(),
            options: HashMap::new(),
            step_last_called: Instant::now(),

            current_time: 0.0,
            newest_data_time: 0.0
        }
    }

    pub fn queue_interpolate(&mut self, key: &str, time: f64, value: f64) {
        let packet = Packet {time, value};

        if self.current_data.contains_key(key) {

            self.data_queue.get_mut(key).unwrap().push(packet);

        } else {

            self.current_data.insert(key.to_string(), packet);

            self.data_queue.insert(key.to_string(), Vec::new());
        }

        if self.newest_data_time == 0.0 {
            self.current_time = time;
        }
        self.newest_data_time = time;
    }

    pub fn step(&mut self) -> Option<SimValue> {
        // No packet received yet
        if self.current_time == 0.0 {return None}
        
        let mut delta = self.step_last_called.elapsed().as_secs_f64();

        self.step_last_called = Instant::now();
        // Should we delay?
        let diff = self.newest_data_time-self.current_time;
        if diff > 1.0 {
            delta += diff - 1.0
        }
    
        // Should delay to let packet queue fill up a bit
        if diff < DELAY_TIME {return None}

        self.current_time += delta;

        let mut return_data = HashMap::new();

        for (key, current) in self.current_data.iter_mut() {
            let queue = self.data_queue.get_mut(key).unwrap();
            let options = self.options.get(key);

            if let Some(value) = set_next_value(current, self.current_time, queue, options) {
                return_data.insert(key.clone(), VarReaderTypes::F64(value));
            }
        }

        return Some(return_data);
    }

    pub fn set_key_options(&mut self, key: &str, options: InterpolateOptions) {
        self.options.insert(key.to_string(), options);
    }

    pub fn reset(&mut self) {
        self.current_data.clear();
        self.step_last_called = Instant::now();
        self.newest_data_time = 0.0;
        self.current_time = 0.0;
    }
}

// Helper functions

fn interpolate_f64(from: f64, to: f64, alpha: f64) -> f64 {
    return from + alpha * (to-from);
}

fn interpolate_f64_degrees(from: f64, to: f64, alpha: f64) -> f64 {
    // If we need to wrap
    if (from - to).abs() > 180.0 {
        // Turning right
        if from < 180.0 && to > 180.0 {
            let from = from + 360.0;
            (from + alpha * -(from - to)) % 360.0
        }
        // Turning left
        else {
            (from + alpha * (to + 360.0 - from)) % 360.0
        }
    }
    else {
        interpolate_f64(from, to, alpha)
    }
}

fn interpolate_f64_degrees_180(from: f64, to: f64, alpha: f64) -> f64 {
    interpolate_f64_degrees(from + 180.0, to + 180.0, alpha) - 180.0
}

fn interpolate_f64_degrees_90(from: f64, to: f64, alpha: f64) -> f64 {
    interpolate_f64_degrees(from + 270.0, to + 270.0, alpha) - 270.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_f64_interpolation() {
        assert_eq!(interpolate_f64(0.0, 10.0, 0.3), 3.0);
        assert_eq!(interpolate_f64(-10.0, 10.0, 0.5), 0.0);
    }

    #[test]
    fn test_heading_rounding() {
        assert_eq!(interpolate_f64_degrees(358.0, 1.0, 0.5) as i32, 359);
        assert_eq!(interpolate_f64_degrees(358.0, 10.0, 0.5) as i32, 4);
        assert_eq!(interpolate_f64_degrees(10.0, 355.0, 0.5) as i32, 2);
        assert_eq!(interpolate_f64_degrees(358.0, 358.0, 0.5) as i32, 358);
    }

    #[test]
    fn test_wrap90() {
        assert_eq!(interpolate_f64_degrees_180(179.0, -179.0, 0.8) as i32, -179);
        assert_eq!(interpolate_f64_degrees_180(-30.0, 30.0, 0.5) as i32, 0);
    }
}