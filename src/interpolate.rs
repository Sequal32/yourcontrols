use crate::bytereader::{StructDataTypes};
use indexmap::IndexMap;
use std::time::{Instant};

struct Record {
    data: IndexMap<String, StructDataTypes>,
    time: Instant
}

pub struct InterpolateStruct {
    last: Option<Record>,
    latest: Option<Record>,
    current: Option<Record>,
    special_floats: Vec<String>,
    interpolation_time: f64,

    add_alpha: f64,
    next_add_alpha: f64
}

impl Default for InterpolateStruct {
    fn default() -> Self {
        Self {last: None, latest: None, current: None, special_floats: vec![], interpolation_time: 0.0, add_alpha: 0.0, next_add_alpha: 0.0}
    }
}

fn get_time() -> Instant {
    return std::time::Instant::now();
}

impl InterpolateStruct {
    pub fn new() -> Self {return Self::default()}

    pub fn record_latest(&mut self, data: IndexMap<String, StructDataTypes>) {
        self.latest = Some(Record {data, time: get_time()});
        self.last = self.latest.take();
        self.interpolation_time = self.latest.as_ref().unwrap().time.duration_since(self.last.as_ref().unwrap().time).as_secs_f64();
        self.add_alpha = self.next_add_alpha;
    }

    pub fn record_current(&mut self, data: IndexMap<String, StructDataTypes>) {
        self.current = Some(Record {data, time: get_time()});
    }

    pub fn add_special_floats(&mut self, data: &mut Vec<String>) {
        self.special_floats.append(data);
    }

    pub fn interpolate(&mut self) -> Option<IndexMap<String, StructDataTypes>> {
        if self.latest.is_none() || self.last.is_none() {return None}

        let mut interpolated = IndexMap::<String, StructDataTypes>::new();

        let current = self.current.as_ref().unwrap();
        let latest = self.latest.as_ref().unwrap();

        let alpha = current.time.duration_since(latest.time).as_secs_f64()/self.interpolation_time + self.add_alpha;
        
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
                        if self.special_floats.contains(key) {
                            value = interpolate_f64_degrees(*current_value, *n, alpha);   
                        } else {
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
    // turning left
    if from < 180.0 && to > 180.0 {
        (from + alpha * -(360.0 - to + from)) % 360.0
    } else if from > 180.0 && to < 180.0 {
        (from + alpha * (360.0 - from + to)) % 360.0
    }
    else {
        return interpolate_f64(from, to, alpha);
    }
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
}
