use std::io::Cursor;

pub const MAX_ENTRY_LEN: usize = 128;

#[derive(Default)]
pub struct GameLog {
    entries: Vec<[u8; MAX_ENTRY_LEN]>,
}

impl GameLog {
    pub fn new_entry(&mut self) -> Cursor<&mut [u8]> {
        self.entries.push([0; MAX_ENTRY_LEN]);
        Cursor::new(&mut self.entries.last_mut().unwrap()[..])
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
