use std::{collections::{HashMap}, time::Instant};
use serde::Deserialize;
use std::collections::VecDeque;

use crate::{util::VarReaderTypes, varreader::SimValue};

const DEFAULT_INTERPOLATION_TIME: f64 = 0.2;
const SKIP_TIME_THRESHOLD: f64 = 2.0;

const SPEED_UP_AT_BUFFER_SIZE_OF: usize = 1;

struct InterpolationData {
    overshoot: f64,
    current_value: f64,
    from_packet: Packet,
    to_packet: Packet,
    time: Instant,
    interpolation_time: f64,
    done: bool
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct InterpolateOptions {
    time: f64,
    to_buffer: usize,
    wrap360: bool,
    wrap180: bool,
    wrap90: bool
}

impl Default for InterpolateOptions {
    fn default() -> Self {
        Self {
            to_buffer: 0,
            time: DEFAULT_INTERPOLATION_TIME,
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

pub struct Interpolate {
    current_data: HashMap<String, InterpolationData>,
    data_queue: HashMap<String, VecDeque<Packet>>,
    options: HashMap<String, InterpolateOptions>,
}

impl Interpolate {
    pub fn new() -> Self {
        Self {
            current_data: HashMap::new(),
            data_queue: HashMap::new(),
            options: HashMap::new(),
        }
    }

    pub fn queue_interpolate(&mut self, key: &str, time: f64, value: f64) {
        let packet = Packet {time, value};

        if self.current_data.contains_key(key) {

            self.data_queue.get_mut(key).unwrap().push_back(packet);

        } else {

            let interpolation_time = match self.options.get(key) {
                Some(o) => o.time,
                None => DEFAULT_INTERPOLATION_TIME
            };

            self.current_data.insert(key.to_string(), InterpolationData {
                overshoot: 0.0,
                from_packet: packet.clone(),
                current_value: value,
                to_packet: packet,
                time: Instant::now(),
                interpolation_time,
                done: false,
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
                    // From is now to
                    std::mem::swap(&mut data.from_packet, &mut data.to_packet);
                    // Calculate time difference between old packet and new packet
                    let mut interpolation_time = next.time-data.from_packet.time;

                    data.to_packet = next;
                    data.time = Instant::now();
                    // Do not interpolate if time exceeds threshold, as aircraft will just stand still interpolating at a very slow rate
                    data.done = interpolation_time > SKIP_TIME_THRESHOLD;

                    //
                    if queue.len() > SPEED_UP_AT_BUFFER_SIZE_OF {
                        // Will make next interpolation faster depending on how many packets over the buffer size it is.
                        interpolation_time = interpolation_time * (SPEED_UP_AT_BUFFER_SIZE_OF as f64)/((queue.len() - SPEED_UP_AT_BUFFER_SIZE_OF) as f64) * 0.5;
                    }
                    data.interpolation_time = interpolation_time;
                }
                continue
            }

            let alpha = data.time.elapsed().as_secs_f64()/data.interpolation_time + data.overshoot;
            
            // If we're done interpolation, do not interpolate anymore until the next request
            if alpha >= 1.0 {
                data.done = true;
                data.current_value = data.to_packet.value;
                data.overshoot = alpha - 1.0;
            } 
            // Interpolate according to options
            if let Some(options) = self.options.get(key) {
                if options.wrap360 {
                    data.current_value = interpolate_f64_degrees(data.from_packet.value, data.to_packet.value, alpha);
                } else if options.wrap180 {
                    data.current_value = interpolate_f64_degrees_180(data.from_packet.value, data.to_packet.value, alpha);
                } else if options.wrap90 {
                    data.current_value = interpolate_f64_degrees_90(data.from_packet.value, data.to_packet.value, alpha);
                } else {
                    data.current_value = interpolate_f64(data.from_packet.value, data.to_packet.value, alpha);
                }
            } else {
                data.current_value = interpolate_f64(data.from_packet.value, data.to_packet.value, alpha);
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