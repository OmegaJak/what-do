use axum_live_view::html;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{app::AppMsg, room_state::RoomState};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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
        let scores = calculate_scores(&room_state.votes);
        let highest_score = scores.first().map_or(0, |(_, score)| *score);
        html! {
            <div>
                <h1>"Results"</h1>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <h2>"Final Results"</h2>
                <div>
                    <ol>
                        for (option, score) in scores.iter() {
                            if *score == highest_score {
                                <h3><li>{format!("{} - {}", option, score)}</li></h3>
                            } else {
                                <li>{format!("{} - {}", option, score)}</li>
                            }
                        }
                    </ol>
                </div>
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
                            <p></p>
                        }
                    </ul>
                </div>
            </div>
        }
    }
}

fn calculate_scores(votes: &[Vec<String>]) -> Vec<(String, usize)> {
    let longest = votes.iter().map(|v| v.len()).max().unwrap_or(0);
    let mut scores: HashMap<String, usize> = HashMap::new();
    for vote in votes {
        for (index, option) in vote.iter().enumerate() {
            let score = longest.checked_sub(index).unwrap();
            scores
                .entry(option.clone())
                .and_modify(|s| *s = *s + score)
                .or_insert(score);
        }
    }

    scores
        .iter()
        .sorted_by_key(|(option, _)| option.to_owned())
        .sorted_by_key(|(_, score)| *score)
        .rev()
        .map(|(option, score)| (option.to_string(), *score))
        .collect()
}
