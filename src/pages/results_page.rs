use axum_live_view::html;
use serde::{Deserialize, Serialize};

use crate::{app::AppMsg, room_state::RoomState};
use std::sync::{Arc, RwLock};

use super::AppPage;

pub struct ResultsPage {
    pub room_code: String,
    pub room_state: Arc<RwLock<RoomState>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ResultsMsg {
    ResultsUpdated,
}

impl AppPage for ResultsPage {
    fn update(
        &mut self,
        msg: crate::app::AppMsg,
        _data: Option<axum_live_view::event_data::EventData>,
        _server_shared_state: &mut crate::ServerwideSharedState,
        _broadcaster: &mut crate::ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>> {
        if let AppMsg::ResultsMsg(msg) = msg {
            match msg {
                ResultsMsg::ResultsUpdated => (), // re-render
            }
        }

        None
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        let room_state = self.room_state.read().unwrap();
        html! {
            <div>
                <h2>"Results"</h2>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <h4>"All Votes"</h4>
                <div>
                    <ul>
                        for vote in room_state.votes.iter() {
                            <li>
                                <ol>
                                    for option in vote.iter() {
                                        <li>{option}</li>
                                    }
                                </ol>
                            </li>
                        }
                    </ul>
                </div>
            </div>
        }
    }
}
