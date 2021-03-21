use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::path::Path;

use yourcontrols_types::{
    ConditionMessage, DatumMessage, Error, InterpolateMessage, InterpolationType, MappingType,
    SyncPermission, VarType, WatchPeriod,
};

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
struct DatumGenerator {
    index_regex: Regex,
}

impl DatumGenerator {
    pub fn new() -> Self {
        Self {
            index_regex: Regex::new(r#":(\d+)"#).unwrap(),
        }
    }

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
    fn get_mapping_datum(
        &self,
        sync_permission: SyncPermission,
        definition: Mapping,
    ) -> Result<DatumMessage, Error> {
        let index = self.get_index_from_var_name(definition.var.get_name());

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
    fn get_key_datum(
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
    fn get_bus_toggle(
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
    fn get_local_var(
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

pub struct DefinitionsParser {
    db: MappingDatabase,
    datums: Vec<DatumMessage>,
    generator: DatumGenerator,
}

impl DefinitionsParser {
    pub fn with_mapping_path(mapping_folder_path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self::with_mappings(MappingDatabase::from_path(
            mapping_folder_path,
        )?))
    }

    pub fn with_mappings(mappings: MappingDatabase) -> Self {
        Self {
            db: mappings,
            datums: Vec::new(),
            generator: DatumGenerator::new(),
        }
    }

    pub fn load_permission_definitions(
        &mut self,
        sync_permission: SyncPermission,
        def: String,
    ) -> Result<(), Error> {
        // Def string has mapping defined in a mapping file
        if let Some(mapping) = self.db.get_mapping_for(&def).cloned() {
            let datum = self.generator.get_mapping_datum(sync_permission, mapping)?;
            self.datums.push(datum);

            return Ok(());
        }

        // Otherwise, add it as a standalone
        let prefix = &def[0..2];
        let def_no_prefix = def[2..].to_string();

        let generator = &self.generator;

        let var_datum = match prefix {
            "K:" => generator.get_key_datum(sync_permission, def_no_prefix)?,
            "B:" => generator.get_bus_toggle(sync_permission, &def_no_prefix)?,
            "L:" | "E:" => generator.get_local_var(sync_permission, def)?,
            _ => return Ok(()),
        };

        self.datums.push(var_datum);

        Ok(())
    }

    fn load_defs(
        &mut self,
        sync_permission: SyncPermission,
        defs: Option<Vec<String>>,
    ) -> Result<(), Error> {
        if let Some(defs) = defs {
            for def in defs {
                self.load_permission_definitions(sync_permission.clone(), def)?
            }
        }

        Ok(())
    }

    fn load_top_down(&mut self, top_down: YamlTopDown) -> Result<(), Error> {
        if let Some(include) = top_down.include {
            for file_path in include {
                self.load_definition_file(file_path)?
            }
        }

        self.load_defs(SyncPermission::Server, top_down.server)?;
        self.load_defs(SyncPermission::Shared, top_down.shared)?;
        self.load_defs(SyncPermission::Init, top_down.init)?;

        // TODO: ignore category

        Ok(())
    }

    pub fn load_definition_file(&mut self, file_path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::open(&file_path)?;
        let yaml: YamlTopDown = serde_yaml::from_reader(file)
            .map_err(|e| Error::YamlError(e, file_path.as_ref().to_string_lossy().to_string()))?;

        self.load_top_down(yaml)
    }

    pub fn get_datums(&self) -> &Vec<DatumMessage> {
        &self.datums
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
struct YamlTopDown {
    ignore: Option<Vec<String>>,
    include: Option<Vec<String>>,
    server: Option<Vec<String>>,
    shared: Option<Vec<String>>,
    init: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mapping {
    mapping: MappingType<String>,
    var: VarType,
    #[serde(flatten)]
    condition: Option<ConditionMessage>,
    interpolate: Option<InterpolationType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERMISSION: SyncPermission = SyncPermission::Init;

    fn get_test_mapping() -> Mapping {
        Mapping {
            mapping: MappingType::ToggleSwitch {
                event_name: "Test".to_string(),
                off_event_name: None,
                switch_on: false,
                event_param: None,
            },
            var: VarType::Named {
                name: "L:Test1:1".to_string(),
            },
            condition: None,
            interpolate: Some(InterpolationType::Wrap180),
        }
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
