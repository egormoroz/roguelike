pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn with(mut self, s: String) -> Self {
        self.entries.push(s);
        self
    }
}