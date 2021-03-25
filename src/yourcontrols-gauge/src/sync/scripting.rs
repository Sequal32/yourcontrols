use std::cell::RefCell;
use std::rc::Rc;

use rhai::{packages::Package, Dynamic, Engine, RegisterFn, Scope, Shared, AST};
use yourcontrols_types::{DatumValue, Result};

use crate::data::{RcSettable, RcVariable, Variable};

thread_local! {
    pub static SCRIPTING_ENGINE: RefCell<ScriptingEngine> = RefCell::new(ScriptingEngine::new());
    static EMPTY_SCOPE: Scope<'static> = Scope::new();
}

pub struct ScriptingEngine {
    scripts: Vec<AST>,
    engine: Engine,
}

impl ScriptingEngine {
    pub fn new() -> Self {
        Self {
            scripts: Vec::new(),
            engine: Engine::new_raw(),
        }
    }

    pub fn setup_engine(&mut self) {
        self.engine
            .register_fn("get", |vec: &mut Vec<RcVariable>, index: i64| {
                vec[index as usize].get()
            });
        self.engine
            .register_fn(
                "set",
                |set: &mut Vec<RcSettable>, index: i64, value: f64| {
                    set[index as usize].set_with_value(value)
                },
            )
            .register_fn("set", |set: &mut Vec<RcSettable>, index: i64| {
                set[index as usize].set()
            });
    }

    pub fn add_script(&mut self, lines: &[&str]) -> Result<()> {
        let mut ast: Option<std::result::Result<AST, rhai::ParseError>> = None;

        EMPTY_SCOPE.with(|x| {
            ast = Some(self.engine.compile_scripts_with_scope(x, lines));
        });

        self.scripts.push(ast.unwrap()?);

        Ok(())
    }

    pub fn run_script(
        &self,
        script_id: usize,
        incoming_value: DatumValue,
        vars: Vec<RcVariable>,
        sets: Vec<RcSettable>,
        params: Vec<Dynamic>,
    ) -> Result<()> {
        let mut scope: Scope = Scope::new();
        scope.push_constant("incoming_value", incoming_value);
        scope.push_constant("vars", vars);
        scope.push_constant("sets", sets);
        scope.push_constant("params", params);

        self.engine
            .eval_ast_with_scope::<()>(&mut scope, &self.scripts[script_id])?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.scripts.clear();
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use rhai::Dynamic;
    use std::rc::Rc;

    use super::*;
    use crate::data::{MockSettable, MockVariable};

    const TEST_SCRIPT: &[&'static str] = &[
        "let test = vars.get(0);",
        "if params[1] {sets.set(0, test + incoming_value + params[0])} else {sets.set(0)};",
    ];

    fn get_engine() -> ScriptingEngine {
        let mut engine = ScriptingEngine::new();
        engine.setup_engine();
        engine
    }

    #[test]
    fn add_script() {
        let mut engine = get_engine();

        assert!(engine.add_script(TEST_SCRIPT).is_ok());
        assert!(engine.scripts.get(0).is_some());
    }

    #[test]
    fn run_calls() {
        let mut engine = get_engine();
        engine
            .add_script(TEST_SCRIPT)
            .expect("should add successfully");

        let mut var = MockVariable::new();
        var.expect_get().times(2).return_const(2.0);

        let mut set = MockSettable::new();
        // With value
        set.expect_set_with_value()
            .once()
            .with(eq(8.0)) // 2 + 1 + 5
            .return_const(());
        // Plain set()
        set.expect_set().once().return_const(());

        let vars: Vec<RcVariable> = vec![Rc::new(var)];
        let sets: Vec<RcSettable> = vec![Rc::new(set)];

        // Set with value (param[1] == true)
        engine
            .run_script(
                0,
                1.0,
                vars.clone(),
                sets.clone(),
                vec![Dynamic::from(5.0), Dynamic::from(true)],
            )
            .expect("should run succesfully");
        // Regular set (param[1] == false)
        engine
            .run_script(
                0,
                1.0,
                vars.clone(),
                sets.clone(),
                vec![Dynamic::from(5.0), Dynamic::from(false)],
            )
            .expect("should run succesfully");
    }
}
