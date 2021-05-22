use crate::error::{Error, Result};
use crate::store::DATABASE;
use yourcontrols_types::{DatumMessage, MappingType, VarType, WatchPeriod};

/// A struct that generates DatumMessages from var strings or mappings.
#[derive(Debug)]
pub struct DatumGenerator {}

impl DatumGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for when key_event is triggered
    /// 2. Trigger the event when a value is passed to it
    fn get_key_datum(&self, key_event: String) -> Result<DatumMessage> {
        // Does not include K:
        Ok(DatumMessage {
            watch_event: Some(key_event),
            mapping: Some(MappingType::Event),
            ..Default::default()
        })
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for when the bus connection is tripped at 16hz
    /// 2. Toggle the connection when a value is passed into it
    fn get_bus_toggle(&self, bus_string: &str) -> Result<DatumMessage> {
        // Should be a connection index and a bus index seperated by a :
        let mut split = bus_string.split(":");

        let bus_index = split.next().ok_or(Error::MissingField("Bus Index"))?;
        let connection_index = split
            .next()
            .ok_or(Error::MissingField("Connection Index"))?;

        let get = format!(
            "{} (>A:BUS LOOKUP INDEX, Number) (A:BUS CONNECTION ON:{}, Bool)",
            bus_index, connection_index
        );
        let set = format!(
            "{} {} (>K:2:ELECTRICAL_BUS_TO_BUS_CONNECTION_TOGGLE)",
            connection_index, bus_index
        );

        Ok(DatumMessage {
            var: Some(DATABASE.add_var(VarType::Calculator { get, set })),
            watch_period: Some(WatchPeriod::Hz16),
            mapping: Some(MappingType::Var),
            ..Default::default()
        })
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for changes to the var at 16hz
    fn get_local_var(&self, var: String) -> Result<DatumMessage> {
        Ok(DatumMessage {
            var: Some(DATABASE.add_var(VarType::Named { name: var })),
            watch_period: Some(WatchPeriod::Hz16),
            mapping: Some(MappingType::Var),
            ..Default::default()
        })
    }

    pub fn get_generated_from_string(&self, string: &str) -> Result<DatumMessage> {
        let prefix = &string[0..2];
        let string_no_prefix = string[2..].to_string();

        match prefix {
            "K:" => self.get_key_datum(string_no_prefix),
            "C:" => self.get_bus_toggle(&string_no_prefix),
            "L:" => self.get_local_var(string_no_prefix),
            _ => Err(Error::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bus_toggle_datum() {
        let generator = DatumGenerator::new();
        let result = generator.get_generated_from_string("C:2:6").unwrap();

        let correct_var = VarType::Calculator {
            get: "2 (>A:BUS LOOKUP INDEX, Number) (A:BUS CONNECTION ON:6, Bool)".to_string(),
            set: "6 2 (>K:2:ELECTRICAL_BUS_TO_BUS_CONNECTION_TOGGLE)".to_string(),
        };

        assert_eq!(DATABASE.get_var(result.var.unwrap()).unwrap(), correct_var);
        assert_eq!(result.watch_period.unwrap(), WatchPeriod::Hz16);
        assert_eq!(result.mapping.unwrap(), MappingType::Var);
    }

    #[test]
    fn get_local_var() {
        let generator = DatumGenerator::new();
        let result = generator.get_generated_from_string("L:TEST VAR").unwrap();

        let correct_var = VarType::Named {
            name: "TEST VAR".to_string(),
        };

        assert_eq!(DATABASE.get_var(result.var.unwrap()).unwrap(), correct_var);
        assert_eq!(result.watch_period, Some(WatchPeriod::Hz16));
    }

    #[test]
    fn get_key_event() {
        let event_string = "K:TOGGLE_GEAR";

        let generator = DatumGenerator::new();
        let result = generator.get_generated_from_string(event_string).unwrap();

        assert_eq!(result.watch_event.unwrap(), "TOGGLE_GEAR");
        assert_eq!(result.mapping.unwrap(), MappingType::Event);
    }
}
