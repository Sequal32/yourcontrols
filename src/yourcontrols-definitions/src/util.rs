use lazy_static::lazy_static;
use regex::Regex;
use rhai::Dynamic;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_yaml::Value;
use yourcontrols_types::{EventMessage, VarType, WatchPeriod};

lazy_static! {
    static ref INDEX_REGEX: Regex = Regex::new(r#"(.+):(\d+)"#).unwrap();
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct YamlTopDown {
    pub ignore: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub server: Option<Vec<Value>>,
    pub shared: Option<Vec<Value>>,
    pub init: Option<Vec<Value>>,
    pub templates: Option<Vec<Template>>,
    pub mappings: Option<Vec<Template>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialTemplate {
    pub name: String,
    #[serde(default)]
    pub use_template: String,
    #[serde(flatten)]
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullTemplate {
    pub name: String,
    pub vars: Vec<VarType>,
    pub sets: Option<Vec<EventMessage>>,
    pub script: Option<String>,
    #[serde(default = "WatchPeriod::default")]
    pub period: WatchPeriod,
    #[serde(default)]
    pub params: Vec<Dynamic>,
    #[serde(flatten)]
    pub misc: Value,
}

impl FullTemplate {
    pub fn get_misc_object<T: DeserializeOwned>(&self, index: &str) -> Option<T> {
        self.misc.get(index).and_then(deserialize_yaml_option)
    }
}

pub fn get_index_from_var_name(var_name: &str) -> Option<(&str, &str)> {
    let captures = INDEX_REGEX.captures(var_name)?;

    Some((captures.get(1)?.as_str(), captures.get(2)?.as_str()))
}

pub fn deserialize_yaml_option<T: DeserializeOwned>(value: &Value) -> Option<T> {
    serde_yaml::from_value(value.clone()).ok()
}

pub fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Mapping(ref mut a), &Value::Mapping(ref b)) => {
            for (k, v) in b {
                if let Some(mapping) = a.get_mut(k) {
                    merge(mapping, v)
                } else {
                    a.insert(k.clone(), Value::Null);
                    merge(a.get_mut(k).unwrap(), v);
                }
            }
        }
        (&mut Value::Sequence(ref mut a), Value::Sequence(ref b)) => {
            for (index, new_value) in b.iter().enumerate() {
                if let Some(value) = a.get_mut(index) {
                    merge(value, new_value);
                }
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Template {
    FullTemplate(FullTemplate),
    PartialTemplate(PartialTemplate),
}

impl Template {
    pub fn get_name(&self) -> &String {
        match self {
            Template::PartialTemplate(t) => &t.name,
            Template::FullTemplate(t) => &t.name,
        }
    }

    pub fn get_first_var_name(&self) -> Option<&str> {
        let t = match self {
            Template::PartialTemplate(t) => t,
            _ => return None,
        };

        match t.value.get("vars")? {
            Value::Sequence(a) => a.get(0)?.get("name")?.as_str(),
            Value::Mapping(a) => a.get(&Value::String(String::from("name")))?.as_str(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_first_var_name() {
        let template: Template = serde_yaml::from_str(
            r#"
            name: ToggleSwitch
            vars:
                -   name: TestName
                -   name: TestName2
        "#,
        )
        .unwrap();

        assert_eq!(template.get_first_var_name(), Some("TestName"))
    }

    #[test]
    fn get_name() {
        let template: Template = serde_yaml::from_str("name: ToggleSwitch").unwrap();
        assert_eq!(template.get_name(), "ToggleSwitch")
    }
}
