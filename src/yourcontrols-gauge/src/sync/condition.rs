use crate::data::RcVariable;
use crate::util::DatumValue;

/// If var is specified, the condition will be applied on the value of the var when `is_satisfied` is called.
/// Otherwise, the condition will be applied on the `incoming_value`.
pub struct Condition {
    var: Option<RcVariable>,
    equals: Option<DatumValue>,
    less_than: Option<DatumValue>,
    greater_than: Option<DatumValue>,
}

impl Condition {
    /// Checks that each non-none condition is satisfied either on the incoming_value or on the var itself.
    pub fn is_satisfied(&self, incoming_value: DatumValue) -> bool {
        let operating_on_value = self
            .var
            .as_ref()
            .map_or(incoming_value, |x| x.borrow().get());

        return self.equals.map_or(true, |x| x == operating_on_value)
            && self.less_than.map_or(true, |x| x < operating_on_value)
            && self.greater_than.map_or(true, |x| x > operating_on_value);
    }
}
