use std::fmt::Debug;

use utils::vec2::{IVec2, UVec2};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferId(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WindowId(pub usize);

#[derive(Debug, Clone)]
pub enum Position {
    Number(usize),
    Start,
    End,
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

pub trait Api {}

#[derive(Debug, Clone)]
pub enum ApiRequest {
    ChangeMode(Mode),
    OpenFile(Option<String>),
    SaveBuffer(Option<BufferId>, Option<String>),
    GetLineCount(Option<BufferId>),
    GetLineLength(Option<BufferId>, usize),
    GetChar(Option<BufferId>, UVec2),
    GetLine(Option<BufferId>, usize),
    GetAllLines(Option<BufferId>),
    GetContent(Option<BufferId>),
    InsertChar(Option<BufferId>, UVec2, char),
    InsertLine(Option<BufferId>, usize, String),
    ReplaceChar(Option<BufferId>, UVec2, char),
    ReplaceLine(Option<BufferId>, usize, String),
    ReplaceAllLines(Option<BufferId>, Vec<String>),
    ReplaceContent(Option<BufferId>, String),
    RemoveChar(Option<BufferId>, UVec2),
    RemoveLine(Option<BufferId>, usize),
    SplitLine(Option<BufferId>, UVec2),
    JoinLines(Option<BufferId>, usize),
    GetPosition(Option<WindowId>),
    MoveBy(Option<WindowId>, IVec2),
    MoveToX(Option<WindowId>, Position),
    MoveToY(Option<WindowId>, Position),
}

#[derive(Debug, Clone)]
pub enum ApiResponse {
    None,
    Number(usize),
    Vec2(UVec2),
    Char(char),
    String(String),
    VecString(Vec<String>),
}

// TODO: 通常プラグイン向けのAPI
pub struct CuprumApi {}

impl CuprumApi {}
