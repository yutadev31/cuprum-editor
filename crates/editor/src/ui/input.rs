use std::collections::HashMap;

use chrono::{DateTime, Duration, Local};
use crossterm::event::{self, Event, KeyModifiers};

use crate::action::{Action, BufferAction, CursorAction, EditorAction, Mode, WindowAction};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Ctrl(char),
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Esc,
}

type Key = Vec<KeyCode>;

#[derive(Debug)]
pub struct Keymap {
    map: HashMap<Key, Action>,
}

impl Keymap {
    /// Register a key sequence to an action
    pub fn reg(&mut self, key: Key, action: Action) {
        self.map.insert(key, action);
    }

    pub fn get(&self, key: &Key) -> Option<&Action> {
        self.map.get(key)
    }
}

impl Default for Keymap {
    fn default() -> Self {
        let mut s = Self {
            map: HashMap::default(),
        };

        // Cursor movement
        s.reg(
            vec![KeyCode::Char('h')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveLeft,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('j')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveDown,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('k')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveUp,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('l')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveRight,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('0')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveToStartOfLine,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('$')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveToEndOfLine,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('g'), KeyCode::Char('g')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveToStartOfBuffer,
            ))),
        );
        s.reg(
            vec![KeyCode::Char('G')],
            Action::Editor(EditorAction::Window(WindowAction::Cursor(
                CursorAction::MoveToEndOfBuffer,
            ))),
        );
        // s.reg(
        //     vec![KeyCode::Char('w')],
        //     Action::Editor(EditorAction::Window(WindowAction::Cursor(
        //         CursorAction::MoveToNextWord,
        //     ))),
        // );
        // s.reg(
        //     vec![KeyCode::Char('b')],
        //     Action::Editor(EditorAction::Window(WindowAction::Cursor(
        //         CursorAction::MoveToPrevWord,
        //     ))),
        // );
        // s.reg(
        //     vec![KeyCode::Char('e')],
        //     Action::Editor(EditorAction::Window(WindowAction::Cursor(
        //         CursorAction::MoveToWordEnd,
        //     ))),
        // );

        // Modes
        s.reg(
            vec![KeyCode::Char('i')],
            Action::Editor(EditorAction::Mode(Mode::Insert)),
        );
        s.reg(
            vec![KeyCode::Char(':')],
            Action::Editor(EditorAction::Mode(Mode::Command)),
        );
        // s.reg(vec![KeyCode::Char('a')], "editor.mode.append");
        // s.reg(vec![KeyCode::Char('I')], "editor.mode.insert-line-start");
        // s.reg(vec![KeyCode::Char('A')], "editor.mode.append-line-end");
        // s.reg(vec![KeyCode::Char('o')], "editor.mode.open-line-below");
        // s.reg(vec![KeyCode::Char('O')], "editor.mode.open-line-above");

        // Editing
        // s.reg(vec![KeyCode::Char('x')], "editor.edit.delete-char");
        // s.reg(vec![KeyCode::Char('X')], "editor.edit.delete-back-char");
        // s.reg(
        //     vec![KeyCode::Char('d'), KeyCode::Char('d')],
        //     "editor.edit.delete-line",
        // );
        // s.reg(vec![KeyCode::Char('D')], "editor.edit.delete-to-line-end");
        // s.reg(
        //     vec![KeyCode::Char('r'), KeyCode::Char('r')],
        //     "editor.edit.replace-char",
        // );
        // s.reg(vec![KeyCode::Char('R')], "editor.edit.replace-mode");
        // s.reg(vec![KeyCode::Char('p')], "editor.edit.paste-after");
        // s.reg(vec![KeyCode::Char('P')], "editor.edit.paste-before");
        // s.reg(
        //     vec![KeyCode::Char('y'), KeyCode::Char('y')],
        //     "editor.edit.yank-line",
        // );
        // s.reg(vec![KeyCode::Char('Y')], "editor.edit.yank-to-line-end");

        // Undo/Redo
        // s.reg(vec![KeyCode::Char('u')], "editor.edit.undo");
        // s.reg(vec![KeyCode::Ctrl('r')], "editor.edit.redo");

        // UI
        // s.reg(vec![KeyCode::Char(':')], "editor.ui.command");
        // s.reg(vec![KeyCode::Char('/')], "editor.ui.search");
        // s.reg(vec![KeyCode::Char('%')], "editor.ui.replace");

        // Leader key
        s.reg(
            vec![KeyCode::Char(' '), KeyCode::Char('w')],
            Action::Editor(EditorAction::Buffer(BufferAction::Save)),
        );

        s
    }
}

#[derive(Debug, Default)]
pub struct InputManager {
    keymap: Keymap,
    key_buffers: Key,
    last_time: Option<DateTime<Local>>,
}

impl InputManager {
    /// CrosstermのキーイベントをアプリケーションのKeyCodeに変換する
    pub fn crossterm_to_app_key(&self, evt: event::Event) -> anyhow::Result<Option<KeyCode>> {
        Ok(match evt {
            Event::Key(evt) => {
                let ch = match evt.code {
                    event::KeyCode::Char(ch) => Some(ch),
                    event::KeyCode::Enter => Some('\n'),
                    event::KeyCode::Tab => Some('\t'),
                    _ => None,
                };

                ch.map(|ch| {
                    if evt.modifiers.contains(KeyModifiers::CONTROL) {
                        KeyCode::Ctrl(ch)
                    } else {
                        KeyCode::Char(ch)
                    }
                })
                .or(match evt.code {
                    event::KeyCode::Up => Some(KeyCode::Up),
                    event::KeyCode::Down => Some(KeyCode::Down),
                    event::KeyCode::Left => Some(KeyCode::Left),
                    event::KeyCode::Right => Some(KeyCode::Right),
                    event::KeyCode::Backspace => Some(KeyCode::Backspace),
                    event::KeyCode::Delete => Some(KeyCode::Delete),
                    event::KeyCode::Esc => Some(KeyCode::Esc),
                    _ => None,
                })
            }
            _ => None,
        })
    }

    pub fn read_event_normal(&mut self) -> anyhow::Result<Option<Action>> {
        let crossterm_key = event::read()?; // 処理の前にキーを読む
        let key = self.crossterm_to_app_key(crossterm_key)?;

        // 500ms以上間隔が空いたらバッファをクリア
        let now = Local::now();
        if let Some(last_time) = self.last_time {
            let duration: Duration = now - last_time;
            if duration.num_milliseconds() > 500 {
                self.key_buffers = Vec::default();
                self.last_time = None;
            }
        }

        // キーが押されたらバッファに追加
        if let Some(code) = key {
            self.key_buffers.push(code);
            self.last_time = Some(now);
        } else {
            return Ok(None);
        }

        // バッファが登録されているアクションにマッチするか確認
        if let Some(action) = self.keymap.get(&self.key_buffers) {
            self.key_buffers = Vec::default();
            self.last_time = None;
            Ok(Some(action.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn read_event_raw(&mut self) -> anyhow::Result<Option<KeyCode>> {
        let crossterm_key = event::read()?; // 処理の前にキーを読む
        let key = self.crossterm_to_app_key(crossterm_key)?;

        Ok(key)
    }
}
