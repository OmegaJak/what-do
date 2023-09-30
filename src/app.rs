use crate::{
    pages::{
        error_page::ErrorPage, ranking_page::RankingMsg, results_page::ResultsMsg,
        room_choice_page::RoomChoiceMsg, veto_page::VetoMsg, AppPage, AppUpdateResponse,
    },
    BroadcastMsg, BroadcastReceiver, ServerwideSharedState,
};
use axum::http::{HeaderMap, Uri};
use axum_live_view::{
    event_data::EventData,
    live_view::{Updated, ViewHandle},
    Html, LiveView,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use tokio::sync::broadcast;
use tracing::warn;

pub struct App {
    shared_state: ServerwideSharedState,
    broadcast_rx_tx: Option<mpsc::Sender<broadcast::Receiver<BroadcastMsg>>>,
    broadcast_rx: Option<BroadcastReceiver>,
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
        page: Box<dyn AppPage + Sync + Send>,
        broadcast_rx: Option<BroadcastReceiver>,
    ) -> Self {
        Self {
            shared_state,
            broadcast_rx_tx: None,
            broadcast_rx,
            current_page: page,
        }
    }
}

impl LiveView for App {
    type Message = AppMsg;

    fn mount(&mut self, _: Uri, _: &HeaderMap, handle: ViewHandle<Self::Message>) {
        let (broadcast_rx_tx, broadcast_rx_rx) = mpsc::channel();
        if let Some(broadcast_rx) = self.broadcast_rx.take() {
            broadcast_rx_tx.send(broadcast_rx).unwrap();
        }
        self.broadcast_rx_tx = Some(broadcast_rx_tx);
        tokio::spawn(async move {
            let recv_result = broadcast_rx_rx.recv();
            match recv_result {
                Ok(mut broadcast_rx) => {
                    while let Ok(broadcast_msg) = broadcast_rx.recv().await {
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
                }
                Err(_) => (),
            }
        });
    }

    fn update(mut self, msg: AppMsg, data: Option<EventData>) -> Updated<Self> {
        let tx = self.broadcast_rx_tx.as_mut().unwrap();
        match self
            .current_page
            .update(msg, data, &mut self.shared_state, tx)
        {
            Ok(AppUpdateResponse {
                next_page,
                js_commands,
            }) => {
                if let Some(page) = next_page {
                    self.current_page = page;
                }

                let mut updated = Updated::new(self);
                if let Some(commands) = js_commands {
                    updated = updated.with_all(commands);
                }

                updated
            }
            Err(e) => {
                self.current_page = Box::new(ErrorPage::new(e));
                Updated::new(self)
            }
        }
    }

    fn render(&self) -> Html<Self::Message> {
        self.current_page.render()
    }
}
