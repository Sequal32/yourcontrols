use crate::bytereader::{StructDataTypes};
use indexmap::IndexMap;
use std::time::{Instant};

struct Record {
    data: IndexMap<String, StructDataTypes>,
    time: f64
}

pub struct InterpolateStruct {
    last: Option<Record>,
    latest: Option<Record>,
    at_latest: Option<Record>,
    current: Option<Record>,
    instant_at_latest: Instant,
    interpolation_time: f64,

    special_floats_regular: Vec<String>,
    special_floats_wrap180: Vec<String>,
    special_floats_wrap90: Vec<String>,

    add_alpha: f64,
    next_add_alpha: f64
}

impl Default for InterpolateStruct {
    fn default() -> Self {
        Self {
            last: None, 
            latest: None, 
            current: None, 
            at_latest: None, 
            instant_at_latest: std::time::Instant::now(), 
            interpolation_time: 0.0, 
            add_alpha: 0.0, 
            next_add_alpha: 0.0,

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
    pub fn new() -> Self {return Self::default()}

    pub fn record_latest(&mut self, data: IndexMap<String, StructDataTypes>, time: f64) {
        self.last = self.latest.take();
        self.at_latest = self.current.take();
        self.latest = Some(Record {data, time: time});
        self.instant_at_latest = std::time::Instant::now();

        if self.last.is_some() {
            self.interpolation_time = self.latest.as_ref().unwrap().time-self.last.as_ref().unwrap().time;
            self.add_alpha = self.next_add_alpha;
        }
    }

    pub fn record_current(&mut self, data: IndexMap<String, StructDataTypes>) {
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

    pub fn get_time_since_last_position(&self) -> Option<f64> {
        if let Some(s) = &self.latest {
            return Some(self.instant_at_latest.elapsed().as_secs_f64());
        }
        return None;
    }

    pub fn interpolate(&mut self) -> Option<IndexMap<String, StructDataTypes>> {
        if self.latest.is_none() || self.at_latest.is_none() || self.interpolation_time == 0.0 {return None}

        let mut interpolated = IndexMap::<String, StructDataTypes>::new();

        let current = self.at_latest.as_ref().unwrap();
        let latest = self.latest.as_ref().unwrap();

        let alpha = self.instant_at_latest.elapsed().as_secs_f64()/self.interpolation_time + self.add_alpha;
        if alpha > 10.0 {return None}
        
        // Account for lag (prevent plane moving backward)
        if alpha > 1.0 {
            self.next_add_alpha = alpha - 1.0;
        }
        
        for (key, value) in &latest.data {
            // Interpolate between next position and current position
            match value {
                StructDataTypes::Bool(_) => {interpolated.insert(key.to_string(), value.clone());},
                StructDataTypes::F64(n) => {
                    if let Some(StructDataTypes::F64(current_value)) = current.data.get(key) {
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
                        interpolated.insert(key.to_string(), StructDataTypes::F64(value));
                    }
                }
                _ => ()
            }
        }

        self.current = Some(Record{data: interpolated.clone(), time: get_time()});

        return Some(interpolated);
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
