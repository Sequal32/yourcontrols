use std::{collections::{HashMap}, time::Instant};
use serde::Deserialize;

use crate::{util::VarReaderTypes, varreader::SimValue};

struct InterpolationData {
    current_value: f64,
    from_value: f64,
    to_value: f64,
    time: Instant,
    done: bool,
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct InterpolateOptions {
    to_buffer: usize,
    wrap360: bool,
    wrap180: bool,
    wrap90: bool
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            to_buffer: 0,
            wrap360: false,
            wrap180: false,
            wrap90: false,
        }
    }
}

pub struct Interpolate {
    current_data: HashMap<String, InterpolationData>,
    options: HashMap<String, InterpolateOptions>,
    interpolation_time: f64
}

impl Interpolate {
    pub fn new(update_rate: f64) -> Self {
        Self {
            current_data: HashMap::new(),
            options: HashMap::new(),
            interpolation_time: update_rate * 2.0
        }
    }

    pub fn queue_interpolate(&mut self, key: &str, value: f64) {
        if let Some(data) = self.current_data.get_mut(key) {

            data.from_value = data.current_value;
            data.to_value = value;
            data.time = Instant::now();
            data.done = false;

        } else {


            self.current_data.insert(key.to_string(), InterpolationData {
                current_value: value,
                from_value: value,
                to_value: value,
                time: Instant::now(),
                done: false
            });
        }
    }

    pub fn step(&mut self) -> SimValue {
        let mut return_data = HashMap::new();

        for (key, data) in self.current_data.iter_mut() {
            if data.done {continue}

            let alpha = data.time.elapsed().as_secs_f64()/self.interpolation_time;
            // If we're done interpolation, do not interpolate anymore until the next request
            data.done = alpha > 2.0;
            // Interpolate according to options
            if let Some(options) = self.options.get(key) {
                if options.wrap360 {
                    data.current_value = interpolate_f64_degrees(data.from_value, data.to_value, alpha);
                } else if options.wrap180 {
                    data.current_value = interpolate_f64_degrees_180(data.from_value, data.to_value, alpha);
                } else if options.wrap90 {
                    data.current_value = interpolate_f64_degrees_90(data.from_value, data.to_value, alpha);
                } else {
                    data.current_value = interpolate_f64(data.from_value, data.to_value, alpha);
                }
            } else {
                data.current_value = interpolate_f64(data.from_value, data.to_value, alpha);
            }
        
            return_data.insert(key.clone(), VarReaderTypes::F64(data.current_value));
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