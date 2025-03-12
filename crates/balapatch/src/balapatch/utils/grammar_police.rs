use regex::Regex;

#[derive(Debug)]
pub struct GrammarPolice {
    text: String,
}

impl GrammarPolice {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    pub fn into_inner(self) -> String {
        self.text
    }

    pub fn correct_capitalization(&mut self) -> &mut Self {
        if self.text.is_empty() {
            return self;
        }

        let mut chars: Vec<char> = self.text.chars().collect();
        chars[0] = chars[0].to_uppercase().nth(0).unwrap();

        let re = Regex::new(r"([.!?])\s*([a-z])").unwrap();
        self.text = re
            .replace_all(
                &chars.into_iter().collect::<String>(),
                |caps: &regex::Captures| format!("{} {}", &caps[1], caps[2].to_uppercase()),
            )
            .to_string();

        self
    }

    pub fn trim_whitespace(&mut self) -> &mut Self {
        let re = Regex::new(r"\s+").unwrap();
        self.text = re.replace_all(self.text.trim(), " ").to_string();

        self
    }

    pub fn ensure_space_after_punctuation(&mut self) -> &mut Self {
        let punctuation: [char; 3] = [',', '.', '!'];

        for ch in self.text.chars() {
            if punctuation.contains(&ch) {}
        }

        self
    }

    pub fn derepeat_words(&mut self) -> &mut Self {
        let re = Regex::new(r"\b(\w+)\s+1\b").unwrap();
        self.text = re.replace_all(&self.text, "$1").to_string();

        self
    }
}
