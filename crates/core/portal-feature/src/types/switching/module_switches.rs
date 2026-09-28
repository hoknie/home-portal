use toml_edit::{DocumentMut, Item};

use super::module::Module;
use crate::types::FieldError;

pub const SECTION: &str = "modules";
pub const LEGACY_KEY: &str = "enabled";
pub const NOT_A_MODULE: &str =
    "is not a module; the modules are proxy, dns, automations, webhooks, users and workflows";
pub const NOT_A_SWITCH: &str = "must be true or false";
pub const NOT_A_TABLE: &str = "must be a table of module names to true or false";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSwitches {
    on: [bool; 7],
}

impl Default for ModuleSwitches {
    fn default() -> ModuleSwitches {
        ModuleSwitches {
            on: Module::ALL.map(Module::default_on),
        }
    }
}

impl ModuleSwitches {
    pub fn resolve(document: &DocumentMut) -> Result<ModuleSwitches, Vec<FieldError>> {
        let mut switches = ModuleSwitches::default();
        let mut errors = Vec::new();
        for module in Module::ALL {
            if let Some(on) = legacy_switch(document, module) {
                switches.on[module.index()] = on;
            }
        }
        match document.get(SECTION) {
            None => {}
            Some(item) => match item.as_table_like() {
                None => errors.push(FieldError::new(SECTION, NOT_A_TABLE)),
                Some(table) => {
                    for (key, value) in table.iter() {
                        let field = format!("{SECTION}.{key}");
                        match (Module::from_name(key), value.as_bool()) {
                            (None, _) => errors.push(FieldError::new(field, NOT_A_MODULE)),
                            (Some(_), None) => errors.push(FieldError::new(field, NOT_A_SWITCH)),
                            (Some(module), Some(on)) => switches.on[module.index()] = on,
                        }
                    }
                }
            },
        }
        if errors.is_empty() {
            Ok(switches)
        } else {
            Err(errors)
        }
    }

    pub fn is_on(self, module: Module) -> bool {
        self.on[module.index()]
    }

    pub fn with(self, module: Module, on: bool) -> ModuleSwitches {
        let mut changed = self;
        changed.on[module.index()] = on;
        changed
    }

    pub fn missing_requirements(self, module: Module) -> Vec<Module> {
        module
            .requires()
            .iter()
            .copied()
            .filter(|required| !self.is_on(*required))
            .collect()
    }

    pub fn required_by(self, module: Module) -> Vec<Module> {
        Module::ALL
            .into_iter()
            .filter(|other| self.is_on(*other) && other.requires().contains(&module))
            .collect()
    }

    pub fn dependency_errors(self) -> Vec<FieldError> {
        Module::ALL
            .into_iter()
            .filter(|module| self.is_on(*module))
            .flat_map(|module| {
                self.missing_requirements(module)
                    .into_iter()
                    .map(move |required| {
                        FieldError::new(
                            format!("{SECTION}.{}", module.name()),
                            format!(
                                "needs the {required} module, which is off; switch {required} on or {module} off",
                                required = required.name(),
                                module = module.name(),
                            ),
                        )
                    })
            })
            .collect()
    }

    pub fn errors(document: &DocumentMut) -> Vec<FieldError> {
        match ModuleSwitches::resolve(document) {
            Ok(switches) => switches.dependency_errors(),
            Err(errors) => errors,
        }
    }
}

fn legacy_switch(document: &DocumentMut, module: Module) -> Option<bool> {
    document
        .get(module.legacy_section()?)
        .and_then(Item::as_table_like)
        .and_then(|table| table.get(LEGACY_KEY))
        .and_then(Item::as_bool)
}
