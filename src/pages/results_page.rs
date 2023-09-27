use axum_live_view::html;
use itertools::Itertools;
use ordinal::Ordinal;
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
        let highest_score = scores.first().map_or(0, |(_, score, _)| *score);
        html! {
            <div>
                <h1>"Results"</h1>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <p>"A score is calculated for each option, weighted by the rankings it received from each voter."</p>
                <h2>"Final Results"</h2>
                <div>
                    <ol>
                        for (option, score, ranks) in scores.iter() {
                            if *score == highest_score {
                                <h3><li>{get_summary_text(option, *score, ranks)}</li></h3>
                            } else {
                                <li>{get_summary_text(option, *score, ranks)}</li>
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

fn get_summary_text(option: &str, score: usize, ranks: &[usize]) -> String {
    format!("{} ({}) - {}", option, get_ranks_text(ranks), score)
}

fn get_ranks_text(ranks: &[usize]) -> String {
    ranks.iter().map(|r| Ordinal(*r).to_string()).join(", ")
}

fn calculate_scores(votes: &[Vec<String>]) -> Vec<(String, usize, Vec<usize>)> {
    let longest = votes.iter().map(|v| v.len()).max().unwrap_or(0);
    let mut scores: HashMap<String, (usize, Vec<usize>)> = HashMap::new();
    for vote in votes {
        for (index, option) in vote.iter().enumerate() {
            let score = longest.checked_sub(index).unwrap();
            let rank = index + 1;
            scores
                .entry(option.clone())
                .and_modify(|(total, ranks)| {
                    *total = *total + score;
                    ranks.push(rank);
                })
                .or_insert((score, vec![rank]));
        }
    }

    scores
        .into_iter()
        .sorted_by_key(|(option, _)| option.clone())
        .sorted_by_key(|(_, (score, _))| *score)
        .rev()
        .map(|(option, (score, ranks))| (option, score, ranks.into_iter().sorted().collect()))
        .collect()
}
