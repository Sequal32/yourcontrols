mod helpers;
mod util;

use helpers::{DatumGenerator, MappingDatabase};
use std::{fs::File, path::Path};
use util::YamlTopDown;
use yourcontrols_types::{DatumMessage, Error, SyncPermission};

pub struct DefinitionsParser {
    db: MappingDatabase,
    datums: Vec<DatumMessage>,
    generator: DatumGenerator,
}

impl DefinitionsParser {
    /// Initializes with mappings from file
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

    /// Generates datums based on loaded mappings or the definition strings themselves.
    fn load_permission_definitions(
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

    /// Loads each def individually.
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

    /// Recursively loads includes and definitions by category.
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

    /// Load all definitions from a file into memory.
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
