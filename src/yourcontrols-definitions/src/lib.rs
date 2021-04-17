mod helpers;
mod store;
mod util;

use helpers::DatumGenerator;
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use store::{map_vec_to_database, DATABASE};
use yourcontrols_types::{
    DatumMessage, Error, MappingArgsMessage, MappingType, Result, SyncPermission, VarId,
};

use util::{get_index_from_var_name, merge, PartialTemplate, Template, YamlTopDown};

type TemplateName = String;

#[derive(Debug)]
struct Mapping {
    sync_permission: SyncPermission,
    template: Template,
}

#[derive(Debug)]
pub struct TemplateDatabase {
    templates: HashMap<TemplateName, Template>,
    scripts: HashMap<String, (VarId, Vec<String>)>,
}

impl TemplateDatabase {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            scripts: HashMap::new(),
        }
    }

    pub fn load_templates(&mut self, templates: Vec<Template>) {
        for template in templates {
            self.templates.insert(template.get_name().clone(), template);
        }
    }

    pub fn load_script(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let mut reader = BufReader::new(File::open(&path)?);
        let mut lines = Vec::new();

        // Read RHAI script lines into a vector
        loop {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => lines.push(buf),
            };
        }

        // Add lines to the database
        self.scripts.insert(
            path.as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            (self.scripts.len(), lines),
        );

        Ok(())
    }

    pub fn get_template(&self, template_name: &str) -> Option<&Template> {
        self.templates.get(template_name)
    }

    pub fn get_script_id(&self, script_name: &str) -> Option<usize> {
        self.scripts.get(script_name).map(|x| x.0.clone())
    }
}

#[derive(Debug)]
pub struct DefinitionsParser {
    templates: TemplateDatabase,
    definitions: Vec<DatumMessage>,
    generator: DatumGenerator,
}

impl DefinitionsParser {
    pub fn new() -> Self {
        Self {
            templates: TemplateDatabase::new(),
            definitions: Vec::new(),
            generator: DatumGenerator::new(),
        }
    }

    fn merge_subtemplates(&mut self, partial: PartialTemplate) -> Result<Template> {
        let mut partial = partial;

        let merging_with = self
            .templates
            .get_template(&partial.use_template)
            .expect("unfinished template");

        let merging_with_value = serde_yaml::to_value(merging_with)?;

        merge(&mut partial.value, &merging_with_value);

        Ok(serde_yaml::from_value(partial.value)?)
    }

    pub fn get_datum_with_template(
        &mut self,
        template: Template,
        index: Option<String>,
    ) -> Result<DatumMessage> {
        match template {
            // Recursively combine subtemplates until we obtain a FullTemplate or a OneTemplate
            Template::PartialTemplate(partial) => {
                let template = self.merge_subtemplates(partial)?;
                self.get_datum_with_template(template, index)
            }
            // Create a DatumMessage from a FullTemplate
            Template::FullTemplate(full) => {
                // Extra fields
                let condition = full.get_misc_object("condition");
                let interpolate = full.get_misc_object("interpolate");

                // Map vars and events to VarIds
                let vars: Vec<usize> = map_vec_to_database(full.vars, |x| DATABASE.add_var(x));
                let watch_var = *vars.get(0).expect("No watch var");

                // Get a script mapping
                let script_id = full
                    .script
                    .and_then(|script_name| self.templates.get_script_id(&script_name));

                let mapping = match script_id {
                    Some(script_id) => {
                        // Preprocess sets
                        let mut sets = full.sets.expect("sets for script");

                        for set in sets.iter_mut() {
                            match set.param.as_deref() {
                                Some("index") => set.param = index.clone(),
                                _ => {}
                            }
                        }

                        let set_ids: Vec<usize> =
                            map_vec_to_database(sets, |x| DATABASE.add_event(x));

                        MappingType::Script(MappingArgsMessage {
                            script_id,
                            vars: vars,
                            sets: set_ids,
                            params: full.params,
                        })
                    }
                    None => MappingType::Var,
                };

                // Compile all info into a DatumMessage
                return Ok(DatumMessage {
                    var: Some(watch_var),
                    watch_period: Some(full.period),
                    mapping: Some(mapping),
                    condition,
                    interpolate,
                    ..Default::default()
                });
            }
        }
    }

    fn get_datum_from_mapping_and_index(
        &mut self,
        name: &str,
        index: Option<String>,
    ) -> Result<DatumMessage> {
        if let Some(mapping) = self.templates.get_template(name).cloned() {
            self.get_datum_with_template(mapping, index)
        } else {
            self.generator.get_generated_from_string(name)
        }
    }

    fn load_sync_templates(
        &mut self,
        templates: Vec<Value>,
        sync_permission: SyncPermission,
    ) -> Result<()> {
        for template in templates {
            let mut datum = match &template {
                Value::String(name) => {
                    match get_index_from_var_name(name) {
                        // Index found, can apply to templates which use event_param: index
                        Some((name, index)) => {
                            self.get_datum_from_mapping_and_index(name, Some(index.to_string()))
                        }
                        // No index found, will not override event_param
                        None => self.get_datum_from_mapping_and_index(name, None),
                    }?
                }
                // Datum is definied as a top level template
                Value::Mapping(_) => {
                    let template: Template = serde_yaml::from_value(template)?;
                    self.get_datum_with_template(template, None)?
                }
                _ => continue,
            };

            datum.sync_permission = Some(sync_permission.clone());

            self.definitions.push(datum)
        }

        Ok(())
    }

    fn load_yaml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
        let file = File::open(&path)?;
        Ok(serde_yaml::from_reader(file)
            .map_err(|x| Error::YamlError(x, path.as_ref().to_string_lossy().to_string()))?)
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let yaml: YamlTopDown = Self::load_yaml(path)?;

        if let Some(include) = yaml.include {
            for include_path in include {
                self.load_file(include_path)?
            }
        }

        if let Some(templates) = yaml.templates {
            self.templates.load_templates(templates);
        }

        if let Some(mappings) = yaml.mappings {
            self.templates.load_templates(mappings);
        }

        if let Some(shared) = yaml.shared {
            self.load_sync_templates(shared, SyncPermission::Shared)?;
        }

        if let Some(init) = yaml.init {
            self.load_sync_templates(init, SyncPermission::Init)?;
        }

        if let Some(server) = yaml.server {
            self.load_sync_templates(server, SyncPermission::Server)?;
        }

        Ok(())
    }

    pub fn load_scripts(&mut self, path: impl AsRef<Path>) -> Result<()> {
        for dir in std::fs::read_dir(path)? {
            let path = match dir {
                Ok(d) => d.path(),
                Err(_) => continue,
            };

            // Is rhai script
            if path.extension().map(|x| x == "rhai").unwrap_or(false) {
                self.templates.load_script(path)?;
            }
        }

        Ok(())
    }
}
