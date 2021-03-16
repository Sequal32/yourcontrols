use yourcontrols_types::DatumValue;

use crate::data::RcVariable;

/// If var is specified, the condition will be applied on the value of the var when `is_satisfied` is called.
/// Otherwise, the condition will be applied on the `incoming_value`.
#[derive(Default)]
pub struct Condition {
    pub var: Option<RcVariable>,
    pub equals: Option<DatumValue>,
    pub less_than: Option<DatumValue>,
    pub greater_than: Option<DatumValue>,
}

impl Condition {
    /// Checks that each non-none condition is satisfied either on the incoming_value or on the var itself.
    pub fn is_satisfied(&self, incoming_value: DatumValue) -> bool {
        let operating_on_value = self
            .var
            .as_ref()
            .map_or(incoming_value, |x| x.borrow().get());

        return self.equals.map_or(true, |x| x == operating_on_value)
            && self.less_than.map_or(true, |x| operating_on_value < x)
            && self.greater_than.map_or(true, |x| operating_on_value > x);
    }
}

#[cfg(test)]
mod tests {
    use super::Condition;

    #[test]
    fn test_incoming_value_condition() {
        let condition = Condition {
            less_than: Some(10.0),
            greater_than: Some(5.0),
            ..Default::default()
        };

        assert!(condition.is_satisfied(7.5));
        assert!(!condition.is_satisfied(3.5));
        assert!(!condition.is_satisfied(15.5));

        let condition = Condition {
            equals: Some(5.0),
            ..Default::default()
        };

        assert!(condition.is_satisfied(5.0));
        assert!(!condition.is_satisfied(2.3));
    }
}
