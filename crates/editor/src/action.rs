use builtin::BuiltinAction;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Builtin(BuiltinAction),
}
