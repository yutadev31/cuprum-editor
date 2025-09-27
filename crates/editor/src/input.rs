use std::collections::HashMap;

use chrono::{DateTime, Duration, Local};
use crossterm::event::{self, Event, KeyModifiers};

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
    map: HashMap<Key, String>,
}

impl Keymap {
    /// Register a key sequence to an action
    pub fn reg(&mut self, key: Key, action: &str) {
        self.map.insert(key, action.to_string());
    }

    pub fn get(&self, key: &Key) -> Option<&String> {
        self.map.get(key)
    }
}

impl Default for Keymap {
    fn default() -> Self {
        let mut s = Self {
            map: HashMap::default(),
        };

        // Cursor movement
        s.reg(vec![KeyCode::Char('h')], "editor.cursor.left");
        s.reg(vec![KeyCode::Char('j')], "editor.cursor.down");
        s.reg(vec![KeyCode::Char('k')], "editor.cursor.up");
        s.reg(vec![KeyCode::Char('l')], "editor.cursor.right");
        s.reg(vec![KeyCode::Char('0')], "editor.cursor.line-start");
        s.reg(vec![KeyCode::Char('$')], "editor.cursor.line-end");
        s.reg(
            vec![KeyCode::Char('g'), KeyCode::Char('g')],
            "editor.cursor.file-start",
        );
        s.reg(vec![KeyCode::Char('G')], "editor.cursor.file-end");
        s.reg(vec![KeyCode::Char('w')], "editor.cursor.next-word");
        s.reg(vec![KeyCode::Char('b')], "editor.cursor.prev-word");
        s.reg(vec![KeyCode::Char('W')], "editor.cursor.word-end");
        s.reg(vec![KeyCode::Char('B')], "editor.cursor.word-start");

        // Modes
        s.reg(vec![KeyCode::Char('i')], "editor.mode.insert");
        s.reg(vec![KeyCode::Char('a')], "editor.mode.append");
        s.reg(vec![KeyCode::Char('I')], "editor.mode.insert-line-start");
        s.reg(vec![KeyCode::Char('A')], "editor.mode.append-line-end");
        s.reg(vec![KeyCode::Char('o')], "editor.mode.open-line-below");
        s.reg(vec![KeyCode::Char('O')], "editor.mode.open-line-above");
        s.reg(vec![KeyCode::Esc], "editor.mode.normal");

        // Editing
        s.reg(vec![KeyCode::Char('x')], "editor.edit.delete-char");
        s.reg(vec![KeyCode::Char('X')], "editor.edit.delete-back-char");
        s.reg(
            vec![KeyCode::Char('d'), KeyCode::Char('d')],
            "editor.edit.delete-line",
        );
        s.reg(vec![KeyCode::Char('D')], "editor.edit.delete-to-line-end");
        s.reg(
            vec![KeyCode::Char('r'), KeyCode::Char('r')],
            "editor.edit.replace-char",
        );
        s.reg(vec![KeyCode::Char('R')], "editor.edit.replace-mode");
        s.reg(vec![KeyCode::Char('p')], "editor.edit.paste-after");
        s.reg(vec![KeyCode::Char('P')], "editor.edit.paste-before");
        s.reg(
            vec![KeyCode::Char('y'), KeyCode::Char('y')],
            "editor.edit.yank-line",
        );
        s.reg(vec![KeyCode::Char('Y')], "editor.edit.yank-to-line-end");

        // Undo/Redo
        s.reg(vec![KeyCode::Char('u')], "editor.edit.undo");
        s.reg(vec![KeyCode::Ctrl('r')], "editor.edit.redo");

        // UI
        s.reg(vec![KeyCode::Char(':')], "editor.ui.command");
        s.reg(vec![KeyCode::Char('/')], "editor.ui.search");
        s.reg(vec![KeyCode::Char('%')], "editor.ui.replace");

        // Leader key
        s.reg(vec![KeyCode::Char(' '), KeyCode::Char('q')], "editor.quit");
        s.reg(
            vec![KeyCode::Char(' '), KeyCode::Char('w')],
            "editor.file.save",
        );
        s.reg(
            vec![KeyCode::Char(' '), KeyCode::Char('o')],
            "editor.file.open",
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
    fn read(&self) -> anyhow::Result<Option<KeyCode>> {
        let evt = event::read()?;
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

    pub fn read_event(&mut self) -> anyhow::Result<Option<String>> {
        let key = self.read()?;

        let now = Local::now();
        if let Some(last_time) = self.last_time {
            let duration: Duration = now - last_time;
            if duration.num_milliseconds() > 500 {
                self.key_buffers = Vec::default();
                self.last_time = None;
            }
        }

        if let Some(code) = key {
            self.key_buffers.push(code);
            self.last_time = Some(now);
        } else {
            return Ok(None);
        }

        if let Some(action) = self.keymap.get(&self.key_buffers) {
            self.key_buffers = Vec::default();
            self.last_time = None;
            Ok(Some(action.to_string()))
        } else {
            Ok(None)
        }
    }
}
