use axum::http::{HeaderMap, Uri};
use axum_live_view::{
    event_data::EventData,
    live_view::{Updated, ViewHandle},
    Html, LiveView,
};
use serde::{Deserialize, Serialize};

use crate::{
    pages::{
        ranking_page::RankingMsg, results_page::ResultsMsg, room_choice_page::RoomChoiceMsg,
        veto_page::VetoMsg, AppPage,
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
    RankingMsg(RankingMsg),
    ResultsMsg(ResultsMsg),
    Submit,
    Update,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormSubmit {
    name: String,
}

impl App {
    pub fn new(
        shared_state: ServerwideSharedState,
        tx: ServerwideBroadcastSender,
        page: Box<dyn AppPage + Sync + Send>,
    ) -> Self {
        Self {
            shared_state,
            tx,
            current_page: page,
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
                    BroadcastMsg::FinishedVetoing => {
                        if handle
                            .send(AppMsg::VetoMsg(VetoMsg::OtherUserFinishedVetoing))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    BroadcastMsg::UpdatedVotes => {
                        if handle
                            .send(AppMsg::ResultsMsg(ResultsMsg::ResultsUpdated))
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
        let (next_page, js_commands) =
            self.current_page
                .update(msg, data, &mut self.shared_state, &mut self.tx);
        if let Some(page) = next_page {
            self.current_page = page;
        }

        let mut updated = Updated::new(self);
        if let Some(commands) = js_commands {
            updated = updated.with_all(commands);
        }

        updated
    }

    fn render(&self) -> Html<Self::Message> {
        self.current_page.render()
    }
}
