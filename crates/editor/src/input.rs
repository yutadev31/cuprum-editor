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
    pub fn register_key(&mut self, key: Key, action: String) {
        self.map.insert(key, action);
    }

    pub fn get_key(&self, key: &Key) -> Option<&String> {
        self.map.get(key)
    }
}

impl Default for Keymap {
    fn default() -> Self {
        let mut this = Self {
            map: HashMap::default(),
        };

        this.register_key(vec![KeyCode::Char('q')], "editor.quit".to_string());
        this.register_key(vec![KeyCode::Char('h')], "editor.cursor.left".to_string());
        this.register_key(vec![KeyCode::Char('j')], "editor.cursor.down".to_string());
        this.register_key(vec![KeyCode::Char('k')], "editor.cursor.up".to_string());
        this.register_key(vec![KeyCode::Char('l')], "editor.cursor.right".to_string());

        this
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
                    _ => None,
                })
            }
            _ => None,
        })
    }

    pub fn read_event(&mut self) -> anyhow::Result<Option<String>> {
        let now = Local::now();
        if let Some(last_time) = self.last_time {
            let duration: Duration = now - last_time;
            if duration.num_milliseconds() > 200 {
                self.key_buffers = Vec::default();
                self.last_time = None;
            }
        }

        if let Some(code) = self.read()? {
            self.key_buffers.push(code);
            self.last_time = Some(now);
        } else {
            return Ok(None);
        }

        if let Some(action) = self.keymap.get_key(&self.key_buffers) {
            self.key_buffers = Vec::default();
            self.last_time = None;
            Ok(Some(action.to_string()))
        } else {
            Ok(None)
        }
    }
}
