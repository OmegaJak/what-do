use crate::{
    app::AppMsg, room_state::RoomState, BroadcastMsg, ServerwideBroadcastSender,
    ServerwideSharedState,
};
use axum_live_view::html;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use super::{deserialize_form, results_page::ResultsPage, AppPage};

pub struct RankingPage {
    pub room_code: String,
    pub room_state: Arc<RwLock<RoomState>>,
}

impl RankingPage {
    pub fn new(room_code: String, room_state: Arc<RwLock<RoomState>>) -> Self {
        Self {
            room_code,
            room_state,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RankingMsg {
    SubmitRanking,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankingFormSubmit {
    ranked_options: String,
}

impl AppPage for RankingPage {
    fn update(
        &mut self,
        msg: AppMsg,
        data: Option<axum_live_view::event_data::EventData>,
        _server_shared_state: &mut ServerwideSharedState,
        broadcaster: &mut ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>> {
        if let AppMsg::RankingMsg(msg) = msg {
            match msg {
                RankingMsg::SubmitRanking => {
                    let ranked_options = deserialize_form::<RankingFormSubmit>(data)
                        .unwrap()
                        .ranked_options;
                    self.room_state
                        .write()
                        .unwrap()
                        .contribute_votes(ranked_options);
                    broadcaster.send(BroadcastMsg::UpdatedVotes).unwrap();
                    return Some(Box::new(ResultsPage {
                        room_code: self.room_code.clone(),
                        room_state: self.room_state.clone(),
                    }));
                }
            }
        }

        None
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        let room_state = self.room_state.read().unwrap();
        html! {
            <div>
                <h2>"Now, rank!"</h2>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <div>
                    <ol id="sortableList">
                        for option in room_state.options.iter().filter(|o| !o.vetoed) {
                            <li>{option.text.clone()}</li> // Can't use data-id here how SortableJS wants you too, doing so produces "unreachable!("unable to find a way to hit this yolo")" in diff.rs
                        }
                    </ol>

                    <form axm-submit={ AppMsg::RankingMsg(RankingMsg::SubmitRanking) }>
                        <input
                            type="hidden"
                            id="sortingOutput"
                            name="ranked_options"
                        />

                        <input type="submit" value="Submit Ranking"/>
                    </form>
                </div>
            </div>
        }
    }
}
