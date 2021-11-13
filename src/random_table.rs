use rand::Rng;

pub struct RandomTable<T: Clone> {
    entries: Vec<(i32, T)>,
    total_weight: i32,
}

impl<T: Clone> RandomTable<T> {
    pub fn new() -> Self { Self { entries: vec![], total_weight: 0 } }

    pub fn add(&mut self, entry: T, weight: i32) {
        assert!(weight >= 0);
        self.total_weight += weight;
        self.entries.push((self.total_weight, entry));
    }

    pub fn extend(&mut self, it: impl Iterator<Item = (T, i32)>) {
        for (entry, weight) in it {
            self.add(entry, weight);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_weight = 0;
    }

    pub fn roll(&self) -> &T {
        assert!(!self.entries.is_empty());

        let w = rand::thread_rng().gen_range(1..=self.total_weight);
        let idx = self.entries.binary_search_by_key(&w, |(x, _)| *x)
            .unwrap_or_else(|x| x);

        &self.entries[idx].1
    }
}
