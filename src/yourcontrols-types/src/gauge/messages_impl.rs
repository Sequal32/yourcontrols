use crate::{MappingType, VarType};

impl MappingType<String> {
    pub fn index_to_u32(self, index: Option<u32>) -> MappingType<u32> {
        match self {
            MappingType::ToggleSwitch {
                event_name,
                off_event_name,
                switch_on,
                ..
            } => MappingType::ToggleSwitch {
                event_name,
                off_event_name,
                switch_on,
                event_param: index,
            },
            MappingType::NumSet {
                event_name,
                swap_event_name,
                multiply_by,
                add_by,
                ..
            } => MappingType::NumSet {
                event_name,
                swap_event_name,
                multiply_by,
                add_by,
                event_param: index,
            },
            MappingType::NumDigitSet {
                inc_events,
                dec_events,
            } => MappingType::NumDigitSet {
                inc_events,
                dec_events,
            },
            MappingType::NumIncrement {
                up_event_name,
                down_event_name,
                increment_amount,
                pass_difference,
            } => MappingType::NumIncrement {
                up_event_name,
                down_event_name,
                increment_amount,
                pass_difference,
            },
            MappingType::Var => MappingType::Var,
            MappingType::Event => MappingType::Event,
        }
    }
}

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
