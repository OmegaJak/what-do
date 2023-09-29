use axum_live_view::html;
use itertools::Itertools;
use ordinal::Ordinal;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppMsg,
    room_state::{FinalVoteTally, RoomState},
};
use std::sync::{Arc, RwLock};

use super::{AppPage, AppUpdateResponse};

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
    ) -> anyhow::Result<AppUpdateResponse> {
        if let AppMsg::ResultsMsg(msg) = msg {
            match msg {
                ResultsMsg::ResultsUpdated => (), // re-render
            }
        }

        Ok((None, None).into())
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        let room_state = self.room_state.read().unwrap();
        let tallies = room_state.tally_votes();
        let highest_score = tallies.first().map_or(0, |t| t.score);
        html! {
            <div>
                <h1>"Results"</h1>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <p>"A score is calculated for each option, weighted by the rankings it received from each voter."</p>
                <h2>"Final Results"</h2>
                <div>
                    <ol>
                        for tally in tallies.iter() {
                            if tally.score == highest_score {
                                <h3><li>{get_summary_text(&tally)}</li></h3>
                            } else {
                                <li>{get_summary_text(&tally)}</li>
                            }
                        }
                    </ol>
                </div>
                <h4>"All Votes"</h4>
                <div>
                    <ul>
                        for votes in room_state.iter_html_displayable_votes() {
                            <li>
                                <ol>
                                    for option in votes {
                                        <li>{option}</li>
                                    }
                                </ol>
                            </li>
                            <p></p>
                        }
                    </ul>
                </div>
            </div>
        }
    }
}

fn get_summary_text(tally: &FinalVoteTally) -> String {
    format!(
        "{} ({}) - {}",
        tally.html_displayable_text,
        get_ranks_text(&tally.ranks),
        tally.score
    )
}

fn get_ranks_text(ranks: &[usize]) -> String {
    ranks.iter().map(|r| Ordinal(*r).to_string()).join(", ")
}
