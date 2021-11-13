use std::io::BufWriter;

pub const MAX_ENTRY_LEN: usize = 128;
pub struct GameLog {
    entries: Vec<[u8; MAX_ENTRY_LEN]>,
}

impl GameLog {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn new_entry(&mut self) -> BufWriter<&mut [u8]> {
        self.entries.push([0; MAX_ENTRY_LEN]);
        BufWriter::new(&mut self.entries.last_mut().unwrap()[..])
    }

    pub fn last_entries(&self, n: usize) -> impl Iterator<Item = &str> {
        let idx = self.entries.len() - n.min(self.entries.len());
        self.entries[idx..].iter()
            .map(|buf| std::str::from_utf8(buf).unwrap())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
