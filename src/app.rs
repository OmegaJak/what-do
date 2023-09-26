use axum::http::{HeaderMap, Uri};
use axum_live_view::{
    event_data::EventData,
    live_view::{Updated, ViewHandle},
    Html, LiveView,
};
use serde::{Deserialize, Serialize};

use crate::{
    pages::{
        room_choice_page::{RoomChoiceMsg, RoomChoicePage},
        veto_page::VetoMsg,
        AppPage,
    },
    BroadcastMsg, ServerwideBroadcastSender, ServerwideSharedState,
};

pub struct App {
    shared_state: ServerwideSharedState,
    tx: ServerwideBroadcastSender,
    current_page: Box<dyn AppPage + Send + Sync>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum AppMsg {
    RoomChoiceMsg(RoomChoiceMsg),
    VetoMsg(VetoMsg),
    Increment,
    Decrement,
    Submit,
    Update,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormSubmit {
    name: String,
}

impl App {
    pub fn new(shared_state: ServerwideSharedState, tx: ServerwideBroadcastSender) -> Self {
        Self {
            shared_state,
            tx,
            current_page: Box::new(RoomChoicePage::new()),
        }
    }
}

impl LiveView for App {
    type Message = AppMsg;

    fn mount(&mut self, _: Uri, _: &HeaderMap, handle: ViewHandle<Self::Message>) {
        let mut rx = self.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(broadcast_msg) = rx.recv().await {
                match broadcast_msg {
                    BroadcastMsg::UpdatedVetos => {
                        if handle
                            .send(AppMsg::VetoMsg(VetoMsg::VetosUpdated))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });
    }

    fn update(mut self, msg: AppMsg, data: Option<EventData>) -> Updated<Self> {
        let next_page = self
            .current_page
            .update(msg, data, &mut self.shared_state, &mut self.tx);
        if let Some(page) = next_page {
            self.current_page = page;
        }
        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        self.current_page.render()
    }
}
