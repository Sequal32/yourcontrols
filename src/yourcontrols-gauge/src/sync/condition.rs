use crate::data::RcVariable;
use crate::util::DatumValue;

pub struct Condition {
    var: Option<RcVariable>,
    equals: Option<DatumValue>,
    less_than: Option<DatumValue>,
    greater_than: Option<DatumValue>,
}

impl Condition {
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
