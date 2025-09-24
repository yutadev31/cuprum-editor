use crossterm::event;

#[derive(Debug, Default)]
pub struct InputManager {}

impl InputManager {
    pub fn read_event(&self) -> anyhow::Result<Option<Action>> {
        match event::read()? {
            event::Event::Key(evt) => match evt.code {
                event::KeyCode::Char('q') => {
                    return Ok(Some(Action::Quit));
                }
                _ => {}
            },
            _ => {}
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum Action {
    Quit,
}
