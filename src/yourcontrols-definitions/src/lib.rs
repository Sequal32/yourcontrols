mod error;
mod helpers;
mod store;
mod util;

use crate::error::{Error, Result};
use helpers::DatumGenerator;
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use store::{map_vec_to_database, EventsRef, VarsRef, DATABASE};
use yourcontrols_types::{
    ConditionMessage, ControlSurfaces, DatumMessage, MappingArgsMessage, MappingType,
    ScriptMessage, SyncPermission, VarId,
};

use util::{get_index_from_var_name, merge, PartialTemplate, Template, YamlTopDown};

type TemplateName = String;

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

    pub fn get_script_messages(&self) -> Vec<ScriptMessage> {
        self.scripts
            .iter()
            .map(|(_, (_, lines))| ScriptMessage {
                lines: lines.clone(),
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct DefinitionsParser {
    templates: TemplateDatabase,
    definitions: Vec<DatumMessage>,
    metadata: HashMap<VarId, DatumMetadata>,
    generator: DatumGenerator,
}

impl DefinitionsParser {
    pub fn new() -> Self {
        Self {
            templates: TemplateDatabase::new(),
            definitions: Vec::new(),
            metadata: HashMap::new(),
            generator: DatumGenerator::new(),
        }
    }

    fn merge_subtemplates(&mut self, partial: PartialTemplate) -> Result<Template> {
        let mut partial = partial;

        let merging_with = self
            .templates
            .get_template(&partial.use_template)
            .expect("missing use template");

        let merging_with_value = serde_yaml::to_value(merging_with)?;

        merge(&mut partial.value, &merging_with_value);

        if let Some(use_template) = partial
            .value
            .get("use_template")
            .map(|x| x.as_str().unwrap())
        {
            // Last chain of use_template - remove so that the template can be parsed as a FullTemplate
            if use_template == "" {
                partial
                    .value
                    .as_mapping_mut()
                    .unwrap()
                    .remove(&Value::String("use_template".to_string()));
            }
        }

        Ok(serde_yaml::from_value(partial.value)?)
    }

    fn get_datum_with_template(
        &mut self,
        template: Template,
        index: Option<String>,
    ) -> Result<(DatumMessage, DatumMetadata)> {
        match template {
            // Recursively combine subtemplates until we obtain a FullTemplate or a OneTemplate
            Template::PartialTemplate(partial) => {
                let template = self.merge_subtemplates(partial)?;
                self.get_datum_with_template(template, index)
            }
            // Create a DatumMessage from a FullTemplate
            Template::FullTemplate(full) => {
                // Extra fields
                let interpolate = full.get_misc_object("interpolate");

                // Map vars and events to VarIds
                let vars: Vec<usize> =
                    map_vec_to_database(full.vars, |x| DATABASE.add_var(x.into()));
                let watch_var = *vars.get(0).expect("No watch var");

                // Handle conditions
                let conditions = full.conditions.map(|conditions| {
                    let mut conditions_result = Vec::new();

                    for condition in conditions {
                        let mut condition_vars = Vec::new();

                        if condition.include_self {
                            condition_vars.extend(vars.iter());
                        }

                        if let Some(vars) = condition.vars {
                            condition_vars.extend(
                                map_vec_to_database(vars, |x| DATABASE.add_var(x.into()))
                                    .into_iter(),
                            )
                        }

                        let script_id = self
                            .templates
                            .get_script_id(&condition.script)
                            .expect("script doesn't exist");

                        conditions_result.push(ConditionMessage {
                            script_id,
                            vars: condition_vars,
                            params: condition.params,
                        });
                    }

                    return conditions_result;
                });

                // Get a script mapping
                let script_id = full
                    .script
                    .and_then(|script_name| self.templates.get_script_id(&script_name));

                let mapping = match script_id {
                    Some(script_id) => {
                        // Preprocess sets
                        //TODO: param_reveresd
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
                return Ok((
                    DatumMessage {
                        var: Some(watch_var),
                        watch_period: Some(full.period),
                        mapping: Some(mapping),
                        conditions,
                        interpolate,
                        ..Default::default()
                    },
                    DatumMetadata {
                        control_surface: full.control_surface,
                    },
                ));
            }
        }
    }

    fn get_datum_from_mapping_and_index(
        &mut self,
        name: &str,
        index: Option<String>,
    ) -> Result<(DatumMessage, DatumMetadata)> {
        if let Some(mapping) = self.templates.get_template(name).cloned() {
            self.get_datum_with_template(mapping, index)
        } else {
            self.generator
                .get_generated_from_string(name)
                .map(|x| (x, DatumMetadata::default()))
        }
    }

    fn load_sync_templates(&mut self, templates: Vec<Value>) -> Result<()> {
        for template in templates {
            let (datum, mut meta_data) = match &template {
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

            self.metadata.insert(self.definitions.len(), meta_data);

            self.definitions.push(datum)
        }

        Ok(())
    }

    fn load_yaml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
        let file = File::open(&path)?;
        Ok(serde_yaml::from_reader(file)
            .map_err(|x| Error::YamlFileError(x, path.as_ref().to_string_lossy().to_string()))?)
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

        if let Some(definitions) = yaml.definitions {
            self.load_sync_templates(definitions);
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

    pub fn get_parsed_datums(&self) -> &Vec<DatumMessage> {
        &self.definitions
    }

    pub fn get_parsed_scripts(&self) -> Vec<ScriptMessage> {
        self.templates.get_script_messages()
    }

    pub fn get_parsed_vars(&self) -> VarsRef {
        DATABASE.get_all_vars()
    }

    pub fn get_parsed_events(&self) -> EventsRef {
        DATABASE.get_all_events()
    }

    /// Gets metadata associated with the specified datum_id
    pub fn get_meta_data_for(&self, datum_id: &VarId) -> Option<&DatumMetadata> {
        self.metadata.get(datum_id)
    }
}

#[derive(Debug, Default)]
pub struct DatumMetadata {
    pub control_surface: Option<ControlSurfaces>,
}
