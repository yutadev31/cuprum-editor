use std::sync::Arc;

use api::{ApiRequest, ApiResponse, Mode, Position};
use tokio::sync::{Mutex, Notify};
use utils::vec2::IVec2;

#[derive(Debug, Default)]
pub struct BuiltinApi {}

impl BuiltinApi {}

#[derive(Debug, Default)]
pub struct Builtin {
    next_index: usize,
    notify: Arc<Notify>,
    pub messages: Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<Option<ApiResponse>>>, ApiRequest)>>>,
}

impl Builtin {
    pub async fn send_message(&mut self, msg: ApiRequest) -> Option<ApiResponse> {
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
        state.clone()
    }

    pub async fn get_messages(
        messages: &Arc<Mutex<Vec<(Arc<Notify>, Arc<Mutex<Option<ApiResponse>>>, ApiRequest)>>>,
    ) -> Vec<(Arc<Notify>, Arc<Mutex<Option<ApiResponse>>>, ApiRequest)> {
        let mut messages = messages.lock().await;

        let queue = messages.clone();
        messages.clear();
        queue
    }

    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub async fn on_action(&mut self, action: BuiltinAction) -> anyhow::Result<()> {
        match action {
            BuiltinAction::Save => {
                self.send_message(ApiRequest::SaveBuffer(None, None)).await;
            }
            BuiltinAction::ChangeMode(mode) => {
                self.send_message(ApiRequest::ChangeMode(mode)).await;
            }
            BuiltinAction::MoveBy(offset) => {
                self.send_message(ApiRequest::MoveBy(None, offset)).await;
            }
            BuiltinAction::MoveToX(pos) => {
                self.send_message(ApiRequest::MoveToX(None, pos)).await;
            }
            BuiltinAction::MoveToY(pos) => {
                self.send_message(ApiRequest::MoveToY(None, pos)).await;
            }
            BuiltinAction::RemoveChar => {
                if let Some(ApiResponse::Vec2(position)) =
                    self.send_message(ApiRequest::GetPosition(None)).await
                {
                    self.send_message(ApiRequest::RemoveChar(None, position))
                        .await;
                }
            }
            BuiltinAction::RemoveLine => {
                if let Some(ApiResponse::Vec2(position)) =
                    self.send_message(ApiRequest::GetPosition(None)).await
                {
                    self.send_message(ApiRequest::RemoveLine(None, position.y))
                        .await;
                }
            }
            BuiltinAction::RemoveSelection => {
                todo!()
            }
            BuiltinAction::OpenLineBelow => {
                if let Some(ApiResponse::Vec2(position)) =
                    self.send_message(ApiRequest::GetPosition(None)).await
                {
                    self.send_message(ApiRequest::InsertLine(None, position.y + 1, String::new()))
                        .await;

                    self.send_message(ApiRequest::MoveBy(None, IVec2::down()))
                        .await;

                    self.send_message(ApiRequest::ChangeMode(Mode::Insert(false)))
                        .await;
                }
            }
            BuiltinAction::OpenLineAbove => {
                if let Some(ApiResponse::Vec2(position)) =
                    self.send_message(ApiRequest::GetPosition(None)).await
                {
                    self.send_message(ApiRequest::InsertLine(None, position.y, String::new()))
                        .await;

                    self.send_message(ApiRequest::ChangeMode(Mode::Insert(false)))
                        .await;
                }
            }
            BuiltinAction::InsertLineStart => {
                self.send_message(ApiRequest::MoveToX(None, Position::Start))
                    .await;

                self.send_message(ApiRequest::ChangeMode(Mode::Insert(false)))
                    .await;
            }
            BuiltinAction::AppendLineEnd => {
                self.send_message(ApiRequest::ChangeMode(Mode::Insert(true)))
                    .await;

                self.send_message(ApiRequest::MoveToX(None, Position::End))
                    .await;
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
