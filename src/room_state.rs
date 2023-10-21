use std::collections::{BinaryHeap, HashMap};

use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use uuid::Uuid;

use crate::BroadcastSender;

const SPLIT_PATTERN: &str = "\n";
const INVALID_VOTE_TEXT: &str = "INVALID VOTE";

pub struct RoomState {
    pub code: String,
    options: LinkedHashMap<Uuid, Option>,
    voting_stage: VotingStage,
    votes: Vec<Vec<Uuid>>,
    broadcast_tx: BroadcastSender,
}

pub struct Option {
    text: String,
    pub vetoed: bool,
    pub id: Uuid,
}

struct VoteTally {
    html_displayable_text: String,
    score: usize,
    ranks: BinaryHeap<usize>,
}

pub struct FinalVoteTally {
    pub html_displayable_text: String,
    pub score: usize,
    pub ranks: Vec<usize>,
}

#[derive(Clone)]
pub enum VotingStage {
    Vetoing,
    Ranking,
}

impl Option {
    pub fn new(text: String) -> Self {
        Self {
            text,
            vetoed: false,
            id: Uuid::new_v4(),
        }
    }

    pub fn get_html_text(&self) -> String {
        ammonia::clean_text(&self.text)
    }
}

impl From<VoteTally> for FinalVoteTally {
    fn from(value: VoteTally) -> Self {
        FinalVoteTally {
            html_displayable_text: value.html_displayable_text,
            score: value.score,
            ranks: value.ranks.into_sorted_vec(),
        }
    }
}

impl RoomState {
    pub fn new(code: String, original_input_text: String, broadcast_tx: BroadcastSender) -> Self {
        let options = parse_options(original_input_text);
        Self {
            code,
            options,
            voting_stage: VotingStage::Vetoing,
            votes: Vec::new(),
            broadcast_tx,
        }
    }

    pub fn get_broadcast_tx(&self) -> BroadcastSender {
        self.broadcast_tx.clone()
    }

    pub fn add_option(&mut self, option: String) {
        if valid_option(&option) && !self.options.iter().any(|(_, o)| o.text == option) {
            let option = Option::new(option);
            self.options.insert(option.id, option);
        }
    }

    pub fn contribute_votes(&mut self, votes_text: String) {
        let votes = if !votes_text.is_empty() {
            parse_votes(votes_text)
        } else {
            self.get_votes_matching_insertion_order()
        };
        self.votes.push(votes);
    }

    pub fn voting_stage(&self) -> VotingStage {
        self.voting_stage.clone()
    }

    pub fn finish_vetoing(&mut self) {
        self.voting_stage = VotingStage::Ranking;
    }

    pub fn veto(&mut self, id: Uuid) {
        self.options.entry(id).and_modify(|o| o.vetoed = true);
    }

    pub fn reset_all_vetos(&mut self) {
        for (_, option) in self.options.iter_mut() {
            option.vetoed = false;
        }
    }

    pub fn iter_options(&self) -> impl Iterator<Item = &Option> {
        self.options.values()
    }

    pub fn iter_html_displayable_votes(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = String> + '_> + '_ {
        self.votes.iter().map(|ids| {
            ids.iter()
                .map(|id| self.get_option_html_displayable_text(id))
        })
    }

    pub fn tally_votes(&self) -> Vec<FinalVoteTally> {
        let longest = self.votes.iter().map(|v| v.len()).max().unwrap_or(0);
        let mut tallies: HashMap<Uuid, VoteTally> = HashMap::new();
        for votes in self.votes.iter() {
            for (index, option_id) in votes.iter().enumerate() {
                let score = longest.checked_sub(index).unwrap();
                let rank = index + 1;
                tallies
                    .entry(option_id.clone())
                    .and_modify(|tally| {
                        tally.score += score;
                        tally.ranks.push(rank);
                    })
                    .or_insert_with(|| VoteTally {
                        html_displayable_text: self.get_option_html_displayable_text(option_id),
                        score,
                        ranks: {
                            let mut heap = BinaryHeap::new();
                            heap.push(rank);
                            heap
                        },
                    });
            }
        }

        tallies
            .into_values()
            .map(|v| FinalVoteTally::from(v))
            .sorted_by_key(|v| v.html_displayable_text.clone())
            .sorted_by_key(|v| v.score)
            .rev()
            .collect()
    }

    fn get_option_html_displayable_text(&self, id: &Uuid) -> String {
        self.options
            .get(id)
            .and_then(|o| Some(o.get_html_text()))
            .unwrap_or(INVALID_VOTE_TEXT.to_string())
    }

    fn get_votes_matching_insertion_order(&mut self) -> Vec<Uuid> {
        self.options
            .iter()
            .filter(|(_, o)| !o.vetoed)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

fn parse_votes(votes_text: String) -> Vec<Uuid> {
    votes_text
        .split(SPLIT_PATTERN)
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect()
}

fn parse_options(options_text: String) -> LinkedHashMap<Uuid, Option> {
    options_text
        .split(SPLIT_PATTERN)
        .unique()
        .map(|s| s.trim().to_string())
        .filter(|s| valid_option(s))
        .map(|s| Option::new(s.to_string()))
        .map(|o| (o.id, o))
        .collect()
}

fn valid_option(option: &str) -> bool {
    !option.is_empty()
}
