use std::{collections::{HashMap}, time::Instant};
use serde::Deserialize;
use std::collections::VecDeque;

use crate::{util::VarReaderTypes, varreader::SimValue};

const DEFAULT_INTERPOLATION_TIME: f64 = 0.2;

struct InterpolationData {
    value: f64,
    from_value: f64,
    target_value: f64,
    time: Instant,
    interpolation_time: f64,
    options: InterpolateOptions,
    done: bool
}

#[derive(Deserialize, Copy, Clone)]
#[serde(default)]
pub struct InterpolateOptions {
    overshoot: f64, // How many seconds to interpolate for after interpolation_time has been reached
    time: f64,
    wrap360: bool,
    wrap180: bool,
    wrap90: bool
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            overshoot: 0.0,
            time: DEFAULT_INTERPOLATION_TIME,
            wrap360: false,
            wrap180: false,
            wrap90: false,
        }
    }
}

pub struct Interpolate {
    current_data: HashMap<String, InterpolationData>,
    data_queue: HashMap<String, VecDeque<f64>>,
    options: HashMap<String, InterpolateOptions>,
    buffer_size: usize
}

impl Interpolate {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            current_data: HashMap::new(),
            data_queue: HashMap::new(),
            options: HashMap::new(),
            buffer_size
        }
    }

    pub fn queue_interpolate(&mut self, key: &str, value: f64) {
        if self.current_data.contains_key(key) {

            self.data_queue.get_mut(key).unwrap().push_back(value);

        } else {

            let options = match self.options.get(key) {
                Some(o) => o.clone(),
                None => {InterpolateOptions::default()}
            };

            self.current_data.insert(key.to_string(), InterpolationData {
                from_value: value,
                value: value,
                target_value: value,
                time: Instant::now(),
                interpolation_time: options.time,
                options: options,
                done: false
            });

            self.data_queue.insert(key.to_string(), VecDeque::new());
        }
    }

    pub fn step(&mut self) -> SimValue {
        let mut return_data = HashMap::new();

        for (key, data) in self.current_data.iter_mut() {
            if data.done {
                let queue = self.data_queue.get_mut(key).unwrap();
                // Interpolate to the next position
                if let Some(next) = queue.pop_front() {
                    data.from_value = data.target_value;
                    data.target_value = next;
                    data.done = false;
                    data.time = Instant::now();

                    if queue.len() > self.buffer_size {
                        data.interpolation_time = data.options.time * (self.buffer_size as f64)/((queue.len() - self.buffer_size) as f64) * 0.5;
                    } else {
                        data.interpolation_time = data.options.time;
                    }
                }
                continue
            }

            let alpha = data.time.elapsed().as_secs_f64()/data.interpolation_time;
            let max_alpha;
            // Determine if we should be interpolating
            let options = self.options.get(key);
            if let Some(_options) = options {
                // TODO: overshoot logic
                max_alpha = 1.0;
            } else {
                max_alpha = 1.0;
            }

            
            // If we're done interpolation, do not interpolate anymore until the next request
            if alpha >= max_alpha {
                data.done = true;
                data.value = data.target_value;
            } else {
                // Interpolate according to options
                if let Some(options) = options {
                    if options.wrap360 {
                        data.value = interpolate_f64_degrees(data.from_value, data.target_value, alpha);
                    } else if options.wrap180 {
                        data.value = interpolate_f64_degrees_180(data.from_value, data.target_value, alpha);
                    } else if options.wrap90 {
                        data.value = interpolate_f64_degrees_90(data.from_value, data.target_value, alpha);
                    } else {
                        data.value = interpolate_f64(data.from_value, data.target_value, alpha);
                    }
                } else {
                    data.value = interpolate_f64(data.from_value, data.target_value, alpha);
                }
            }
        
            return_data.insert(key.clone(), VarReaderTypes::F64(data.value));
        }

        return return_data;
    }

    pub fn set_key_options(&mut self, key: &str, options: InterpolateOptions) {
        self.options.insert(key.to_string(), options);
    }

    pub fn reset(&mut self) {
        self.current_data.clear();
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