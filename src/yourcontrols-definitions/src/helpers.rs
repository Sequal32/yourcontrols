use std::{
    collections::HashMap,
    fs::{read_dir, File},
    path::Path,
};

use regex::Regex;
use yourcontrols_types::{
    DatumMessage, Error, InterpolateMessage, MappingType, SyncPermission, VarType, WatchPeriod,
};

use crate::util::Mapping;

/// Loads mappings by iterating through the contents of a folder.
/// Stores all mappings into a HashMap where the key is the var_name and the value is Mapping.
pub struct MappingDatabase {
    mapping_database: HashMap<String, Mapping>,
}

impl MappingDatabase {
    pub fn new() -> Self {
        Self {
            mapping_database: HashMap::new(),
        }
    }

    pub fn from_path(folder_path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut new = Self::new();

        new.load_mappings(folder_path)?;

        Ok(new)
    }

    fn add_mapping(&mut self, mapping: Mapping) {
        self.mapping_database
            .insert(mapping.var.get_name().clone(), mapping);
    }

    fn add_file_mapping(&mut self, file_path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::open(file_path.as_ref())?;

        let yaml: Vec<Mapping> = serde_yaml::from_reader(file)
            .map_err(|e| Error::YamlError(e, file_path.as_ref().to_string_lossy().to_string()))?;

        for mapping in yaml {
            self.add_mapping(mapping)
        }
        Ok(())
    }

    pub fn load_mappings(&mut self, folder_path: impl AsRef<Path>) -> Result<(), Error> {
        let dir = read_dir(folder_path)?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.extension().unwrap() != "yaml" {
                continue;
            }

            self.add_file_mapping(path)?
        }

        Ok(())
    }

    pub fn get_mapping_for(&self, name: &str) -> Option<&Mapping> {
        self.mapping_database.get(name)
    }
}

/// A struct that generates DatumMessages from var strings or mappings.
pub struct DatumGenerator {
    index_regex: Regex,
}

impl DatumGenerator {
    pub fn new() -> Self {
        Self {
            index_regex: Regex::new(r#":(\d+)"#).unwrap(),
        }
    }

    /// Extracts the index from a var name.
    ///
    /// Example: A:PLANE LATITUDE:1 would have an index of 1.
    fn get_index_from_var_name(&self, var_name: &str) -> Option<u32> {
        self.index_regex
            .captures(var_name)?
            .get(1)?
            .as_str()
            .parse()
            .ok()
    }

    /// Generates a datum message to...
    ///
    /// 1. Listen for changes in var every Frame if being interpolated, otherwise at 16hz
    /// 2. Assign a mapping to be executed when a value triggers it (with an optional conditional)
    pub fn get_mapping_datum(
        &self,
        sync_permission: SyncPermission,
        definition: Mapping,
    ) -> Result<DatumMessage, Error> {
        // Check that the mapping explicitly wants an index
        let index = if definition.mapping.has_index() {
            self.get_index_from_var_name(definition.var.get_name())
        } else {
            None
        };

        let period = if definition.interpolate.is_some() {
            WatchPeriod::Frame
        } else {
            WatchPeriod::Hz16
        };

        let interpolation = definition.interpolate.as_ref().map(|x| InterpolateMessage {
            calculator: definition.var.get_set_calculator(),
            interpolate_type: x.clone(),
        });

        let mapping = definition.mapping.index_to_u32(index);

        Ok(DatumMessage {
            var: Some(definition.var),
            watch_period: Some(period),
            condition: definition.condition,
            interpolate: interpolation,
            mapping: Some(mapping),
            sync_permission: Some(sync_permission),
            ..Default::default()
        })
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for when key_event is triggered
    /// 2. Trigger the event when a value is passed to it
    pub fn get_key_datum(
        &self,
        sync_permission: SyncPermission,
        key_event: String,
    ) -> Result<DatumMessage, Error> {
        // Does not include K:
        Ok(DatumMessage {
            watch_event: Some(key_event),
            mapping: Some(MappingType::Event),
            sync_permission: Some(sync_permission),
            ..Default::default()
        })
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for when the bus connection is tripped at 16hz
    /// 2. Toggle the connection when a value is passed into it
    pub fn get_bus_toggle(
        &self,
        sync_permission: SyncPermission,
        bus_string: &str,
    ) -> Result<DatumMessage, Error> {
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
            var: Some(VarType::Calculator { get, set }),
            watch_period: Some(WatchPeriod::Hz16),
            mapping: Some(MappingType::Var),
            sync_permission: Some(sync_permission),
            ..Default::default()
        })
    }

    /// Generates a datum to...
    ///
    /// 1. Listen for changes to the var at 16hz
    pub fn get_local_var(
        &self,
        sync_permission: SyncPermission,
        var: String,
    ) -> Result<DatumMessage, Error> {
        Ok(DatumMessage {
            var: Some(VarType::Named { name: var }),
            watch_period: Some(WatchPeriod::Hz16),
            sync_permission: Some(sync_permission),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yourcontrols_types::InterpolationType;

    const PERMISSION: SyncPermission = SyncPermission::Init;

    fn get_test_mapping_custom(
        event_name: &str,
        var_name: &str,
        event_param: Option<String>,
    ) -> Mapping {
        Mapping {
            mapping: MappingType::ToggleSwitch {
                event_name: event_name.to_string(),
                off_event_name: None,
                switch_on: false,
                event_param,
            },
            var: VarType::Named {
                name: var_name.to_string(),
            },
            condition: None,
            interpolate: Some(InterpolationType::Wrap180),
        }
    }

    fn get_test_mapping() -> Mapping {
        get_test_mapping_custom("Test", "L:Test1:1", Some("index".to_string()))
    }

    #[test]
    fn index_name_from_var() {
        let parser = DatumGenerator::new();
        assert_eq!(
            parser.get_index_from_var_name("A:PLANE LATITUDE:1"),
            Some(1)
        );
        assert!(parser.get_index_from_var_name("A:PLANE LATITUDE").is_none())
    }

    #[test]
    fn get_mapping_datum() {
        let generator = DatumGenerator::new();
        let test_mapping = get_test_mapping();
        let result = generator
            .get_mapping_datum(PERMISSION, test_mapping.clone())
            .unwrap();

        assert_eq!(result.condition, None);
        assert_eq!(result.sync_permission.unwrap(), PERMISSION);
        assert_eq!(
            result.interpolate.unwrap(),
            InterpolateMessage {
                calculator: "(>L:Test1:1)".to_string(), // Should've gotten calculated
                interpolate_type: test_mapping.interpolate.unwrap()
            }
        );
        assert_eq!(
            result.mapping.unwrap(),
            test_mapping.mapping.index_to_u32(Some(1)) // Should've gotten converted
        );
        assert_eq!(result.var.unwrap(), test_mapping.var);
        assert_eq!(result.watch_event, None);
        assert_eq!(result.watch_period.unwrap(), WatchPeriod::Frame); // Is interpolate
    }

    #[test]
    fn get_mapping_datum_index_not_defined() {
        let generator = DatumGenerator::new();
        let test_mapping = get_test_mapping_custom("Test", "L:Test:1", None);
        let result = generator
            .get_mapping_datum(PERMISSION, test_mapping)
            .unwrap();

        if let Some(MappingType::ToggleSwitch { event_param, .. }) = result.mapping {
            assert!(event_param.is_none())
        }
    }

    #[test]
    fn get_bus_toggle_datum() {
        let generator = DatumGenerator::new();
        let result = generator.get_bus_toggle(PERMISSION, "2:6").unwrap();

        let correct_var = VarType::Calculator {
            get: "2 (>A:BUS LOOKUP INDEX, Number) (A:BUS CONNECTION ON:6, Bool)".to_string(),
            set: "6 2 (>K:2:ELECTRICAL_BUS_TO_BUS_CONNECTION_TOGGLE)".to_string(),
        };

        assert_eq!(result.var.unwrap(), correct_var);
        assert_eq!(result.watch_period.unwrap(), WatchPeriod::Hz16);
        assert_eq!(result.mapping.unwrap(), MappingType::Var);
        assert_eq!(result.sync_permission.unwrap(), PERMISSION);
    }

    #[test]
    fn get_local_var() {
        let generator = DatumGenerator::new();
        let result = generator
            .get_local_var(PERMISSION, "L:TEST VAR".to_string())
            .unwrap();

        let correct_var = VarType::Named {
            name: "L:TEST VAR".to_string(),
        };

        assert_eq!(result.sync_permission.unwrap(), PERMISSION);
        assert_eq!(result.var.unwrap(), correct_var);
        assert_eq!(result.watch_period, Some(WatchPeriod::Hz16));
    }

    #[test]
    fn get_key_event() {
        let event_string = "K:TOGGLE_GEAR".to_string();

        let generator = DatumGenerator::new();
        let result = generator
            .get_key_datum(PERMISSION, event_string.clone())
            .unwrap();

        assert_eq!(result.watch_event.unwrap(), event_string);
        assert_eq!(result.mapping.unwrap(), MappingType::Event);
        assert_eq!(result.sync_permission.unwrap(), PERMISSION);
    }
}
