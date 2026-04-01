use serde::Serialize;
use std::collections::HashMap;

use crate::util::InDataTypes;
use yourcontrols_types::{AllNeedSync, Error, VarReaderTypes};

#[derive(Serialize)]
pub struct EmulatorVarInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub var_type: String,
    pub value: Option<f64>,
}

#[derive(Clone, Copy)]
pub enum EmulatorVarSource {
    Aircraft,
    Local,
}

pub struct EmulatorState {
    vars: HashMap<String, EmulatorVarEntry>,
    last_known_values: HashMap<String, VarReaderTypes>,
}

struct EmulatorVarEntry {
    name: String,
    var_type: InDataTypes,
    source: EmulatorVarSource,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            last_known_values: HashMap::new(),
        }
    }

    pub fn register_var(&mut self, name: &str, var_type: InDataTypes, source: EmulatorVarSource) {
        let id = make_var_id(name, var_type);
        self.vars.entry(id).or_insert(EmulatorVarEntry {
            name: name.to_string(),
            var_type,
            source,
        });
    }

    pub fn record_last_known(&mut self, name: &str, value: VarReaderTypes) {
        self.last_known_values.insert(name.to_string(), value);
    }

    pub fn update_from_sync(&mut self, data: &AllNeedSync) {
        for (name, value) in &data.avars {
            self.record_last_known(name, *value);
        }

        for (name, value) in &data.lvars {
            self.record_last_known(name, *value);
        }
    }

    pub fn get_vars(&self) -> Vec<EmulatorVarInfo> {
        let mut vars: Vec<_> = self
            .vars
            .iter()
            .map(|(id, entry)| {
                let var_type = var_type_to_string(entry.var_type).to_string();
                EmulatorVarInfo {
                    id: id.clone(),
                    name: entry.name.clone(),
                    display_name: format!("{} ({})", entry.name, var_type),
                    var_type,
                    value: self
                        .last_known_values
                        .get(&entry.name)
                        .map(var_reader_to_f64),
                }
            })
            .collect();

        vars.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        vars
    }

    pub fn get_var_value(&self, id: &str) -> Option<EmulatorVarInfo> {
        let entry = self.vars.get(id)?;
        let var_type = var_type_to_string(entry.var_type).to_string();

        Some(EmulatorVarInfo {
            id: id.to_string(),
            name: entry.name.clone(),
            display_name: format!("{} ({})", entry.name, var_type),
            var_type,
            value: self
                .last_known_values
                .get(&entry.name)
                .map(var_reader_to_f64)
                .or(Some(0.0)),
        })
    }

    pub fn apply_value(
        &mut self,
        id: &str,
        value: f64,
        current_sync: &mut AllNeedSync,
    ) -> Result<(), Error> {
        let entry = self
            .vars
            .get(id)
            .ok_or_else(|| Error::MissingMapping(id.to_string()))?;

        let typed_value = match entry.var_type {
            InDataTypes::I32 => VarReaderTypes::I32(value as i32),
            InDataTypes::I64 => VarReaderTypes::I64(value as i64),
            InDataTypes::F64 => VarReaderTypes::F64(value),
            InDataTypes::Bool => VarReaderTypes::Bool(value != 0.0),
        };

        match entry.source {
            EmulatorVarSource::Aircraft => {
                current_sync.avars.insert(entry.name.clone(), typed_value);
            }
            EmulatorVarSource::Local => {
                current_sync.lvars.insert(entry.name.clone(), typed_value);
            }
        }

        let name = entry.name.clone();
        self.record_last_known(&name, typed_value);
        Ok(())
    }
}

fn var_type_to_string(var_type: InDataTypes) -> &'static str {
    match var_type {
        InDataTypes::I32 => "i32",
        InDataTypes::I64 => "i64",
        InDataTypes::F64 => "f64",
        InDataTypes::Bool => "bool",
    }
}

fn var_reader_to_f64(value: &VarReaderTypes) -> f64 {
    match value {
        VarReaderTypes::F64(v) => *v,
        VarReaderTypes::I32(v) => *v as f64,
        VarReaderTypes::I64(v) => *v as f64,
        VarReaderTypes::Bool(v) => {
            if *v {
                1.0
            } else {
                0.0
            }
        }
    }
}

fn make_var_id(name: &str, var_type: InDataTypes) -> String {
    format!("{}|{}", name, var_type_to_string(var_type))
}
