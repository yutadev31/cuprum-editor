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

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert(bool),
    Command,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert(false) => "INSERT",
            Mode::Insert(true) => "INSERT (APPEND)",
            Mode::Command => "COMMAND",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum WindowAction {
    Cursor(CursorAction),
    Edit(EditAction),
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
pub enum EditAction {
    RemoveChar,
    RemoveLine,
    OpenLineBelow,
    OpenLineAbove,
    InsertLineStart,
    AppendLineEnd,
}

#[derive(Debug, Clone)]
pub enum BufferAction {
    Save,
}
