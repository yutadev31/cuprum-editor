use std::sync::Arc;

use api::{ApiRequest, ApiResponse, CuprumApi, CuprumApiProvider, Mode, Position};
use tokio::sync::{Mutex, Notify};
use utils::vec2::IVec2;

#[derive(Debug, Default)]
pub struct BuiltinApiProvider {
    next_index: usize,
    notify: Arc<Notify>,
    pub messages: Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)>>>,
}

impl BuiltinApiProvider {
    pub async fn get_messages(
        messages: &Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)>>>,
    ) -> Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)> {
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
    messages: Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)>>>,
}

impl Builtin {
    pub fn new() -> Self {
        let provider = BuiltinApiProvider::default();
        Self {
            notify: provider.get_notify(),
            messages: provider.messages.clone(),
            api: CuprumApi::new(provider),
        }
    }

    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub fn get_messages(
        &self,
    ) -> Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<ApiResponse>>, ApiRequest)>>> {
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
                todo!()
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
