use std::sync::Arc;

use api::{
    CuprumApi, CuprumApiProvider, CuprumApiRequestKind, CuprumApiResponseKind, Mode, Position,
};
use tokio::sync::{Mutex, Notify};
use utils::vec2::IVec2;

pub type Messages = Vec<(
    Arc<Notify>,
    Arc<Mutex<Option<CuprumApiResponseKind>>>,
    CuprumApiRequestKind,
)>;

#[derive(Debug, Default)]
pub struct BuiltinApiProvider {
    next_index: usize,
    notify: Arc<Notify>,
    pub messages: Arc<Mutex<Messages>>,
}

impl BuiltinApiProvider {
    pub async fn get_messages(messages: &Arc<Mutex<Messages>>) -> Messages {
        let mut messages = messages.lock().await;

        let queue = messages.clone();
        messages.clear();
        queue
    }

    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }
}

impl CuprumApiProvider for BuiltinApiProvider {
    async fn send_message(
        &mut self,
        msg: CuprumApiRequestKind,
    ) -> anyhow::Result<Option<CuprumApiResponseKind>> {
        let notify = Arc::new(Notify::new());
        let state = Arc::new(Mutex::new(None));
        {
            let mut messages = self.messages.lock().await;
            messages.push((notify.clone(), state.clone(), msg));
        }
        self.next_index += 1;
        self.notify.notify_one();
        notify.notified().await;
        let state = state.lock().await;
        Ok(state.clone())
    }
}

#[derive(Debug)]
pub struct Builtin {
    api: CuprumApi<BuiltinApiProvider>,
    notify: Arc<Notify>,
    messages: Arc<Mutex<Messages>>,
}

impl Builtin {
    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub fn get_messages(&self) -> Arc<Mutex<Messages>> {
        self.messages.clone()
    }

    pub async fn on_action(&mut self, action: BuiltinAction) -> anyhow::Result<()> {
        match action {
            BuiltinAction::Save => {
                self.api.save_buffer(None, None).await?;
            }
            BuiltinAction::ChangeMode(mode) => {
                self.api.change_mode(mode).await?;
            }
            BuiltinAction::MoveBy(offset) => {
                self.api.move_by(None, offset).await?;
            }
            BuiltinAction::MoveToX(pos) => {
                self.api.move_to_x(None, pos).await?;
            }
            BuiltinAction::MoveToY(pos) => {
                self.api.move_to_y(None, pos).await?;
            }
            BuiltinAction::RemoveChar => {
                let pos = self.api.get_cursor(None).await?;
                self.api.remove(None, pos, pos).await?;
            }
            BuiltinAction::RemoveLine => {
                let pos = self.api.get_cursor_vec2(None).await?;
                self.api.remove_line(None, pos.y).await?;
            }
            BuiltinAction::RemoveSelection => {
                let cursor = self.api.get_cursor(None).await?;
                let visual_start = self.api.get_visual_start(None).await?;

                let (left, right, _x) = if cursor < visual_start {
                    (cursor, visual_start, true)
                } else {
                    (visual_start, cursor, false)
                };

                self.api.remove(None, left, right).await?;

                // TODO _xがtrueの場合、カーソルのY罪表を消した分戻す
                self.api.move_to(None, visual_start).await?;

                self.api.change_mode(Mode::Normal).await?;
            }
            BuiltinAction::OpenLineBelow => {
                let pos = self.api.get_cursor_vec2(None).await?;
                self.api.insert_line(None, pos.y + 1, String::new()).await?;
                self.api.move_by(None, IVec2::down()).await?;
                self.api.change_mode(Mode::Insert(false)).await?;
            }
            BuiltinAction::OpenLineAbove => {
                let pos = self.api.get_cursor_vec2(None).await?;
                self.api.insert_line(None, pos.y, String::new()).await?;
                self.api.change_mode(Mode::Insert(false)).await?;
            }
            BuiltinAction::InsertLineStart => {
                self.api.move_to_x(None, Position::Start).await?;
                self.api.change_mode(Mode::Insert(false)).await?;
            }
            BuiltinAction::AppendLineEnd => {
                self.api.change_mode(Mode::Insert(true)).await?;
                self.api.move_to_x(None, Position::End).await?;
            }
        }

        Ok(())
    }
}

impl Default for Builtin {
    fn default() -> Self {
        let provider = BuiltinApiProvider::default();
        Self {
            notify: provider.get_notify(),
            messages: provider.messages.clone(),
            api: CuprumApi::new(provider),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinAction {
    Save,
    ChangeMode(Mode),
    MoveBy(IVec2),
    MoveToX(Position),
    MoveToY(Position),
    RemoveChar,
    RemoveLine,
    RemoveSelection,
    OpenLineBelow,
    OpenLineAbove,
    InsertLineStart,
    AppendLineEnd,
}
