use std::{collections::HashMap, time::{Instant}};
use std::collections::VecDeque;

use crate::util::VarReaderTypes;

struct Record {
    data: HashMap<String, VarReaderTypes>,
    time: f64
}

pub struct InterpolateStruct {
    latest: Option<Record>,
    at_latest: Option<Record>,
    current: Option<Record>,
    instant_at_latest: Instant,
    interpolation_time: f64,

    packet_queue: VecDeque<Record>,
    buffer_size: usize,

    special_floats_regular: Vec<String>,
    special_floats_wrap180: Vec<String>,
    special_floats_wrap90: Vec<String>,
}

impl Default for InterpolateStruct {
    fn default() -> Self {
        Self {
            latest: None, 
            current: None, 
            at_latest: None, 
            instant_at_latest: std::time::Instant::now(), 
            interpolation_time: 0.0, 

            buffer_size: 3,

            packet_queue: VecDeque::new(),
            special_floats_regular: vec![],
            special_floats_wrap90: vec![],
            special_floats_wrap180: vec![],
        }
    }
}

fn get_time() -> f64 {
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
}

impl InterpolateStruct {
    pub fn new(buffer_size: usize) -> Self {
        return Self {
            buffer_size,
            .. Default::default()
        }
    }

    fn to_next(&mut self) {
        let last = self.latest.take();

        self.latest = self.packet_queue.pop_back();
        self.at_latest = self.current.take();     

        if last.is_some() {
            // Calculate time to next position by taking the diff of the latest packet to the previous one
            self.instant_at_latest = std::time::Instant::now();
            self.interpolation_time = self.latest.as_ref().unwrap().time-last.unwrap().time;
        }
    }

    pub fn record_latest(&mut self, data: HashMap<String, VarReaderTypes>, time: f64) {
        self.packet_queue.push_front(Record {data, time});
        // Initial packet setting
        if self.latest.is_none() {
            self.to_next();
        }
    }

    pub fn record_current(&mut self, data: HashMap<String, VarReaderTypes>) {
        self.current = Some(Record {data, time: get_time()});
    }

    pub fn add_special_floats_regular(&mut self, data: &mut Vec<String>) {
        self.special_floats_regular.append(data);
    }

    pub fn add_special_floats_wrap90(&mut self, data: &mut Vec<String>) {
        self.special_floats_wrap90.append(data);
    }

    pub fn add_special_floats_wrap180(&mut self, data: &mut Vec<String>) {
        self.special_floats_wrap180.append(data);
    }

    pub fn get_time_since_last_position(&self) -> f64 {
        return self.instant_at_latest.elapsed().as_secs_f64();
    }

    pub fn interpolate(&mut self) -> Option<HashMap<String, VarReaderTypes>> {
        if self.latest.is_none() || self.at_latest.is_none() {return None}

        let mut interpolated = HashMap::<String, VarReaderTypes>::new();

        let current = self.at_latest.as_ref().unwrap();
        let latest = self.latest.as_ref().unwrap();

        let elapsed = self.get_time_since_last_position();
        let mut alpha = elapsed/self.interpolation_time;

        if alpha > 1.0 {
            alpha = 1.0
        }
        
        for (key, value) in &latest.data {
            // Interpolate between next position and current position
            match value {
                VarReaderTypes::Bool(_) => {interpolated.insert(key.to_string(), value.clone());},
                VarReaderTypes::F64(n) => {
                    if let Some(VarReaderTypes::F64(current_value)) = current.data.get(key) {
                        let value: f64;
                        if self.special_floats_regular.contains(key) {
                            value = interpolate_f64_degrees(*current_value, *n, alpha);   
                        } else if self.special_floats_wrap90.contains(key) {
                            value = interpolate_f64_degrees_90(*current_value, *n, alpha);   
                        } else if self.special_floats_wrap180.contains(key) {
                            value = interpolate_f64_degrees_180(*current_value, *n, alpha);   
                        }
                        else {
                            value = interpolate_f64(*current_value, *n, alpha);
                        }
                        interpolated.insert(key.to_string(), VarReaderTypes::F64(value));
                    }
                }
                _ => ()
            }
        }

        self.current = Some(Record {data: interpolated.clone(), time: get_time()});

        // If the packet queue is overflowing, we want to get to the next position ASAP
        // If we reached the next position in time, we can go to the next packet if the buffer has one
        if alpha >= 1.0 && self.packet_queue.len() > 0 {
            self.to_next();
            // Catch up in the queue
            if self.packet_queue.len() > self.buffer_size {
                self.interpolation_time *= (self.buffer_size + 1) as f64/(self.packet_queue.len() - self.buffer_size) as f64 * 0.5
            }
        }

        return Some(interpolated);
    }

    pub fn reset(&mut self) {
        self.latest = None;
        self.current = None;
        self.at_latest = None;
        self.packet_queue.clear();
    }

    pub fn overloaded(&self) -> bool {
        return self.packet_queue.len() > self.buffer_size + 15
    }
}

pub fn interpolate_f64(from: f64, to: f64, alpha: f64) -> f64 {
    return from + alpha * (to-from);
}

pub fn interpolate_f64_degrees(from: f64, to: f64, alpha: f64) -> f64 {
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

pub fn interpolate_f64_degrees_180(from: f64, to: f64, alpha: f64) -> f64 {
    interpolate_f64_degrees(from + 180.0, to + 180.0, alpha) - 180.0
}

pub fn interpolate_f64_degrees_90(from: f64, to: f64, alpha: f64) -> f64 {
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
