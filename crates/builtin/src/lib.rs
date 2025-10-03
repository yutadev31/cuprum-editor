use std::sync::Arc;

use api::{ApiRequest, ApiResponse, Mode, Position};
use tokio::sync::{Mutex, Notify};
use utils::vec2::IVec2;

#[derive(Debug, Default)]
pub struct BuiltinApi {
    next_index: usize,
    queue: Vec<(Arc<Notify>, Arc<Mutex<Option<ApiResponse>>>, ApiRequest)>,
}

impl BuiltinApi {
    pub async fn send_message(&mut self, msg: ApiRequest) {
        let notify = Arc::new(Notify::new());
        let state = Arc::new(Mutex::new(None));
        self.queue.push((notify.clone(), state.clone(), msg));
        self.next_index += 1;
    }

    pub fn get_messages(
        &mut self,
    ) -> Vec<(Arc<Notify>, Arc<Mutex<Option<ApiResponse>>>, ApiRequest)> {
        let queue = self.queue.clone();
        self.queue.clear();
        queue
    }
}

#[derive(Debug, Default)]
pub struct Builtin {
    pub api: BuiltinApi,
}

impl Builtin {
    pub async fn on_action(&mut self, action: BuiltinAction) -> anyhow::Result<()> {
        match action {
            BuiltinAction::Save => {
                self.api
                    .send_message(ApiRequest::SaveBuffer(None, None))
                    .await;
            }
            BuiltinAction::ChangeMode(mode) => {
                self.api.send_message(ApiRequest::ChangeMode(mode)).await;
            }
            BuiltinAction::MoveBy(offset) => {
                self.api
                    .send_message(ApiRequest::MoveBy(None, offset))
                    .await;
            }
            BuiltinAction::MoveToX(pos) => {
                self.api.send_message(ApiRequest::MoveToX(None, pos)).await;
            }
            BuiltinAction::MoveToY(pos) => {
                self.api.send_message(ApiRequest::MoveToY(None, pos)).await;
            }
            BuiltinAction::RemoveChar => {}
            BuiltinAction::RemoveLine => {}
            BuiltinAction::OpenLineBelow => {}
            BuiltinAction::OpenLineAbove => {}
            BuiltinAction::InsertLineStart => {}
            BuiltinAction::AppendLineEnd => {}
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
    OpenLineBelow,
    OpenLineAbove,
    InsertLineStart,
    AppendLineEnd,
}
