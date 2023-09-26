pub struct RoomState {
    pub code: String,
    pub options: Vec<Option>,
}

pub struct Option {
    pub text: String,
    pub vetoed: bool,
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
        Self { code, options }
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

fn parse_options(options_text: String) -> Vec<Option> {
    options_text
        .split("\n")
        .map(|s| Option::new(s.to_string()))
        .collect()
}
