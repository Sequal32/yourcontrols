use yourcontrols_types::InterpolationType;

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
}
