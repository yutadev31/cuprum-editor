#[derive(Debug, Clone)]
pub enum Action {
    Editor(EditorAction),
}

#[derive(Debug, Clone)]
pub enum EditorAction {
    Quit,
    Mode(Mode),
    Window(WindowAction),
    Buffer(BufferAction),
}

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum WindowAction {
    Cursor(CursorAction),
}

#[derive(Debug, Clone)]
pub enum CursorAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToStartOfLine,
    MoveToEndOfLine,
    MoveToStartOfBuffer,
    MoveToEndOfBuffer,
}

#[derive(Debug, Clone)]
pub enum BufferAction {
    Save,
}
