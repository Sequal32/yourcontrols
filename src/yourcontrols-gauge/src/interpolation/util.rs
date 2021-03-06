use std::collections::VecDeque;
use std::time::SystemTime;

pub enum InterpolationType {
    Default,
    DefaultConstant,
    Wrap360,
    Wrap180,
    Wrap90,
    Invert,
    InvertConstant,
}

impl InterpolationType {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::InvertConstant | Self::DefaultConstant => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct Packet {
    pub value: f64,
    pub time: f64,
    pub current: f64,
}

pub struct Data {
    pub current_packet: Packet,
    pub did_init: bool,
    pub last_received_time: f64,
    pub interpolate_type: InterpolationType,
    pub calculator: String,
    pub queue: VecDeque<Packet>,
}

impl Data {
    pub fn new(calculator: String, interpolate_type: InterpolationType) -> Self {
        Self {
            current_packet: Packet::default(),
            did_init: false,
            last_received_time: 0.0,
            interpolate_type,
            calculator,
            queue: VecDeque::new(),
        }
    }

    pub fn calculate_next_value(&mut self, current_time: f64) {
        if self.queue.is_empty() {
            return;
        }

        for index in 0..self.queue.len() {
            let next_packet = self.queue.get(index).cloned().unwrap();

            // Packet is in the future
            if next_packet.time - current_time > 0.0 {
                // How far we are into interpolating from the "current" packet to the next packet
                let mut alpha = (current_time - self.current_packet.time)
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
                        alpha = (current_time - new_current.time)
                            / (next_packet.time - new_current.time);
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
    }
}

pub fn interpolate(from: f64, to: f64, alpha: f64) -> f64 {
    return from + alpha * (to - from);
}

pub fn interpolate_degrees(from: f64, to: f64, alpha: f64) -> f64 {
    let mut from = from;

    if (from - to).abs() > 180.0 {
        if from < 180.0 && to > 180.0 {
            from = from + 360.0;
            return (from + alpha * -(from - to)) % 360.0;
        } else {
            return (from + alpha * (to + 360.0 - from)) % 360.0;
        }
    } else {
        return interpolate(from, to, alpha);
    }
}

pub fn interpolate_degrees180(from: f64, to: f64, alpha: f64) -> f64 {
    return interpolate_degrees(from + 180.0, to + 180.0, alpha) - 180.0;
}

pub fn interpolate_degrees90(from: f64, to: f64, alpha: f64) -> f64 {
    return interpolate_degrees(from + 270.0, to + 270.0, alpha) - 270.0;
}

pub fn interpolate_value(
    from: f64,
    to: f64,
    alpha: f64,
    interpolate_type: &InterpolationType,
) -> f64 {
    match interpolate_type {
        InterpolationType::Default | InterpolationType::DefaultConstant => {
            interpolate(from, to, alpha)
        }
        InterpolationType::Wrap360 => interpolate_degrees(from, to, alpha),
        InterpolationType::Wrap180 => interpolate_degrees180(from, to, alpha),
        InterpolationType::Wrap90 => interpolate_degrees90(from, to, alpha),
        InterpolationType::Invert | InterpolationType::InvertConstant => {
            interpolate(from, to, alpha) * -1.0
        }
    }
}

pub fn get_time_as_seconds() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_interpolate() {
        assert_eq!(
            interpolate_value(0.0, 100.0, 0.8, &InterpolationType::Default),
            80.0
        );
        assert_eq!(
            interpolate_value(-16384.0, 16384.0, 0.35, &InterpolationType::Default).round(),
            -4915.0
        );
    }

    #[test]
    fn test_360_interpolate() {
        assert_eq!(
            interpolate_value(190.0, 360.0, 0.5, &InterpolationType::Wrap360),
            275.0
        );
        assert_eq!(
            interpolate_value(-10.0, 20.0, 0.5, &InterpolationType::Wrap360),
            5.0
        );
        assert_eq!(
            interpolate_value(320.0, 20.0, 0.25, &InterpolationType::Wrap360),
            335.0
        );
        assert_eq!(
            interpolate_value(358.0, 2.0, 0.5, &InterpolationType::Wrap360),
            0.0
        );
    }

    #[test]
    fn test_180_interpolate() {
        assert_eq!(
            interpolate_value(-85.0, -90.0, 0.5, &InterpolationType::Wrap180),
            -87.5
        );
        assert_eq!(
            interpolate_value(-150.0, 85.0, 0.5, &InterpolationType::Wrap180),
            147.5
        );
    }

    #[test]
    fn test_90_interpolate() {
        assert_eq!(
            interpolate_value(25.0, 85.0, 0.4, &InterpolationType::Wrap90),
            49.0
        );
        assert_eq!(
            interpolate_value(85.0, -75.0, 0.5, &InterpolationType::Wrap90),
            5.0
        );
    }

    #[test]
    fn test_calculate_next_packet() {
        let mut data = Data::new(String::new(), InterpolationType::Default);
        data.queue.push_back(Packet {
            value: 0.0,
            time: 0.0,
            current: 0.0,
        });

        data.queue.push_back(Packet {
            value: 5.0,
            time: 1.0,
            current: 0.0,
        });

        data.queue.push_back(Packet {
            value: 20.0,
            time: 3.0,
            current: 0.0,
        });

        data.queue.push_back(Packet {
            value: 0.0,
            time: 13.0,
            current: 0.0,
        });

        // In sequence
        data.calculate_next_value(0.0);
        assert_eq!(data.current_packet.current, 0.0);
        data.calculate_next_value(0.5);
        assert_eq!(data.current_packet.current, 2.5);
        data.calculate_next_value(1.0);
        assert_eq!(data.current_packet.current, 5.0);
        data.calculate_next_value(2.0);
        assert_eq!(data.current_packet.current.round(), 13.0);

        // Uh oh skip forward!
        data.calculate_next_value(5.0);
        assert_eq!(data.current_packet.current, 16.0);
    }
}
