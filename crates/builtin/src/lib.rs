use std::sync::Arc;

use api::{ApiRequest, ApiResponse, CuprumApi, CuprumApiProvider, Mode, Position};
use tokio::sync::{Mutex, Notify};
use utils::vec2::{IVec2, UVec2};

pub type Messages = Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)>;

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
    async fn send_message(&mut self, msg: ApiRequest) -> anyhow::Result<ApiResponse> {
        let notify = Arc::new(Notify::new());
        let state = Arc::new(Mutex::new(ApiResponse::None));
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
                let pos = self.api.get_position(None).await?;
                self.api.remove_char(None, pos).await?;
            }
            BuiltinAction::RemoveLine => {
                let pos = self.api.get_position(None).await?;
                self.api.remove_line(None, pos.y).await?;
            }
            BuiltinAction::RemoveSelection => {
                let cursor = self.api.get_position(None).await?;
                let visual_start = self.api.get_visual_start(None).await?;

                let (left, right, x) = if cursor < visual_start {
                    (cursor, visual_start, true)
                } else {
                    (visual_start, cursor, false)
                };

                let z = if left.y != right.y {
                    for _ in 0..right.x + 1 {
                        self.api.remove_char(None, UVec2::new(0, right.y)).await?;
                    }

                    let z = right.y - left.y - 1;

                    for _ in left.y + 1..right.y {
                        self.api.remove_line(None, left.y + 1).await?;
                    }

                    let line_len = self.api.get_line_length(None, left.y).await?;
                    for _ in left.x..line_len {
                        self.api.remove_char(None, UVec2::new(0, left.y)).await?;
                    }

                    self.api.join_lines(None, left.y).await?;

                    if x { z } else { 0 }
                } else {
                    for _ in left.x..right.x {
                        self.api
                            .remove_char(None, UVec2::new(left.x, left.y))
                            .await?;
                    }

                    0
                };

                self.api
                    .move_to_y(None, Position::Number(visual_start.y - z))
                    .await?;
                self.api
                    .move_to_x(None, Position::Number(visual_start.x))
                    .await?;

                self.api.change_mode(Mode::Normal).await?;
            }
            BuiltinAction::OpenLineBelow => {
                let pos = self.api.get_position(None).await?;
                self.api.insert_line(None, pos.y + 1, String::new()).await?;
                self.api.move_by(None, IVec2::down()).await?;
                self.api.change_mode(Mode::Insert(false)).await?;
            }
            BuiltinAction::OpenLineAbove => {
                let pos = self.api.get_position(None).await?;
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
