use itertools::Itertools;

const SPLIT_PATTERN: &str = "\n";

pub struct RoomState {
    pub code: String,
    pub options: Vec<Option>,
    voting_stage: VotingStage,
    pub votes: Vec<Vec<String>>,
}

pub struct Option {
    pub text: String,
    pub vetoed: bool,
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
        }
    }
}

impl RoomState {
    pub fn new(code: String, original_input_text: String) -> Self {
        let options = parse_options(original_input_text);
        Self {
            code,
            options,
            voting_stage: VotingStage::Vetoing,
            votes: Vec::new(),
        }
    }

    pub fn contribute_votes(&mut self, votes_text: String) {
        let votes = if !votes_text.is_empty() {
            parse_votes(votes_text)
        } else {
            self.options
                .iter()
                .filter(|o| !o.vetoed)
                .map(|o| o.text.to_string())
                .collect()
        };
        self.votes.push(votes);
    }

    pub fn voting_stage(&self) -> VotingStage {
        self.voting_stage.clone()
    }

    pub fn finish_vetoing(&mut self) {
        self.voting_stage = VotingStage::Ranking;
    }

    pub fn veto_all(&mut self, option_to_veto: &str) {
        for option in self.options.iter_mut() {
            if option.text == option_to_veto {
                option.vetoed = true;
            }
        }
    }

    pub fn reset_all_vetos(&mut self) {
        for option in self.options.iter_mut() {
            option.vetoed = false;
        }
    }
}

fn parse_votes(votes_text: String) -> Vec<String> {
    votes_text
        .split(SPLIT_PATTERN)
        .map(|s| s.to_string())
        .collect()
}

fn parse_options(options_text: String) -> Vec<Option> {
    options_text
        .split(SPLIT_PATTERN)
        .unique()
        .filter(|s| !s.is_empty())
        .map(|s| Option::new(s.to_string()))
        .collect()
}
