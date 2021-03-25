use std::cell::RefCell;

use rhai::{Dynamic, Engine, RegisterFn, Scope, AST};
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
            .register_fn("get", |var: &RcVariable| var.get())
            .register_fn("set", |var: &RcSettable| var.set())
            .register_fn("set", |var: &RcSettable, value: DatumValue| {
                var.set_with_value(value)
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
    ) {
        let mut scope: Scope = Scope::new();
        scope.push_constant("incoming_value", incoming_value);
        scope.push_constant("vars", vars);
        scope.push_constant("sets", sets);

        self.engine
            .eval_ast_with_scope::<()>(&mut scope, &self.scripts[script_id]);
    }

    pub fn reset(&mut self) {
        self.scripts.clear();
    }
}
