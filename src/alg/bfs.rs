use std::collections::VecDeque;

use smallvec::SmallVec;

#[derive(Default)]
pub struct BFS<N: Clone> {
    frontier: VecDeque<(N, i32)>,
}

impl<N: Clone> BFS<N> {
    pub fn search<G, St, Gt, A, I>(
        &mut self, 
        sources: I,
        graph: &mut G, 
        mut set: St, 
        mut get: Gt,
        mut adjacent: A,
    )
    where
        St: FnMut(&mut G, &N, i32),
        Gt: FnMut(&mut G, &N) -> i32,
        A: FnMut(&mut G, &N) -> SmallVec<[N; 8]>,
        I: IntoIterator<Item = N>,
    {
        self.frontier.clear();
        for s in sources {
            set(graph, &s, 0);
            self.frontier.push_back((s, 0));
        }

        while let Some((n, x)) = self.frontier.pop_front() {
            for n in adjacent(graph, &n) {
                if get(graph, &n) >= 0 { continue; }
                set(graph, &n, x + 1);
                self.frontier.push_back((n, x + 1));
            }
        }
    }

    pub fn search_until<G, St, Gt, A, I, D>(
        &mut self, 
        sources: I,
        graph: &mut G, 
        mut set: St, 
        mut get: Gt,
        mut adjacent: A,
        mut dst: D,
    ) -> Option<N>
    where
        St: FnMut(&mut G, &N, i32),
        Gt: FnMut(&mut G, &N) -> i32,
        A: FnMut(&mut G, &N) -> SmallVec<[N; 8]>,
        I: IntoIterator<Item = N>,
        D: FnMut(&mut G, &N) -> bool,
    {
        self.frontier.clear();
        for s in sources {
            set(graph, &s, 0);
            self.frontier.push_back((s, 0));
        }

        while let Some((n, x)) = self.frontier.pop_front() {
            if dst(graph, &n) { return Some(n); }

            for n in adjacent(graph, &n) {
                if get(graph, &n) >= 0 { continue; }
                set(graph, &n, x + 1);
                self.frontier.push_back((n, x + 1));
            }
        }

        None
    }
}

