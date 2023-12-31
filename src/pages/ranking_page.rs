use crate::{
    app::AppMsg, room_state::RoomState, BroadcastMsg, BroadcastSender, ServerwideSharedState,
};
use axum_live_view::{html, js_command};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use super::{deserialize_form, results_page::ResultsPage, AppPage, AppUpdateResponse};

pub struct RankingPage {
    pub room_code: String,
    pub room_state: Arc<RwLock<RoomState>>,
    pub broadcast_tx: BroadcastSender,
}

impl RankingPage {
    pub fn new(
        room_code: String,
        room_state: Arc<RwLock<RoomState>>,
        broadcast_tx: BroadcastSender,
    ) -> Self {
        Self {
            room_code,
            room_state,
            broadcast_tx,
        }
    }

    fn get_results_page_response(&mut self) -> anyhow::Result<AppUpdateResponse> {
        Ok(AppUpdateResponse {
            next_page: Some(Box::new(ResultsPage {
                room_code: self.room_code.clone(),
                room_state: self.room_state.clone(),
            })),
            js_commands: Some(vec![js_command::history_push_state(
                format!("/room/{}/results", self.room_code).parse().unwrap(),
            )]),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RankingMsg {
    SubmitRanking,
    JustViewResults,
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
        _broadcast_rx_tx: &mut crate::BroadcastReceiverSender,
    ) -> anyhow::Result<AppUpdateResponse> {
        if let AppMsg::RankingMsg(msg) = msg {
            match msg {
                RankingMsg::SubmitRanking => {
                    let ranked_options =
                        deserialize_form::<RankingFormSubmit>(data)?.ranked_options;
                    self.room_state
                        .write()
                        .unwrap()
                        .contribute_votes(ranked_options);
                    self.broadcast_tx.send(BroadcastMsg::UpdatedVotes)?;
                    return self.get_results_page_response();
                }
                RankingMsg::JustViewResults => {
                    return self.get_results_page_response();
                }
            }
        }

        Ok((None, None).into())
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        let room_state = self.room_state.read().unwrap();
        html! {
            <div>
                <h1>"Now, rank!"</h1>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <p>"Each voter can rank their preferences individually by dragging and dropping the items in the list below."</p>
                <button style="font-size:0.75rem;" axm-click={AppMsg::RankingMsg(RankingMsg::JustViewResults)}>"View Results w/o Voting"</button>
                <div>
                    <ol id="sortableList">
                        for option in room_state.iter_options().filter(|o| !o.vetoed) {
                            <li style="cursor:move;" option-id={option.id.as_simple().to_string()}>{option.get_html_text()}</li> // Can't use data-id here how SortableJS wants you too, doing so produces "unreachable!("unable to find a way to hit this yolo")" in diff.rs
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
