use crate::{MappingArgsMessage, VarType};

impl VarType {
    pub fn get_name(&self) -> &String {
        match self {
            VarType::WithUnits { name, .. } => name,
            VarType::Named { name, .. } => name,
            VarType::Calculator { get, .. } => get,
        }
    }

    pub fn get_set_calculator(&self) -> String {
        match self {
            VarType::WithUnits { name, units, .. } => format!("(>{}, {})", name, units),
            VarType::Named { name } => format!("(>{})", name),
            VarType::Calculator { set, .. } => set.clone(),
        }
    }
}

impl PartialEq for MappingArgsMessage {
    fn eq(&self, other: &Self) -> bool {
        self.script_id == other.script_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_type_name() {
        assert_eq!(
            VarType::WithUnits {
                name: "Test".to_string(),
                units: "Hola".to_string(),
                index: None
            }
            .get_name(),
            "Test"
        );

        assert_eq!(
            VarType::Named {
                name: "Test".to_string(),
            }
            .get_name(),
            "Test"
        );

        assert_eq!(
            VarType::Calculator {
                get: "Test".to_string(),
                set: "None".to_string()
            }
            .get_name(),
            "Test"
        );
    }
}
