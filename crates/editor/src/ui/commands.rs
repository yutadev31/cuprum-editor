use std::collections::HashMap;

use builtin::BuiltinAction;

use crate::action::Action;

#[derive(Debug)]
pub struct CommandMap {
    map: HashMap<String, Action>,
}

impl CommandMap {
    /// Register a command name to an action
    pub fn reg(&mut self, name: &str, action: Action) {
        self.map.insert(name.to_string(), action);
    }

    pub fn get(&self, name: &str) -> Option<&Action> {
        self.map.get(name)
    }
}

impl Default for CommandMap {
    fn default() -> Self {
        let mut s = Self {
            map: HashMap::default(),
        };

        s.reg("q", Action::Quit);
        s.reg("w", Action::Builtin(BuiltinAction::Save));

        s
    }
}
