use std::collections::VecDeque;

use smallvec::SmallVec;

#[derive(Default)]
pub struct BFS<N: Clone> {
    frontier: VecDeque<(N, i32)>,
}

impl<N: Clone> BFS<N> {
    pub fn search<G, St, Gt, A>(
        &mut self, 
        start: N,
        graph: &mut G, 
        mut set: St, 
        mut get: Gt,
        mut adjacent: A
    )
    where
        St: FnMut(&mut G, &N, i32),
        Gt: FnMut(&mut G, &N) -> i32,
        A: FnMut(&mut G, &N) -> SmallVec<[N; 8]>,
    {
        set(graph, &start, 0);
        self.frontier.clear();
        self.frontier.push_back((start, 0));

        while let Some((n, x)) = self.frontier.pop_front() {
            for n in adjacent(graph, &n) {
                if get(graph, &n) >= 0 { continue; }
                set(graph, &n, x + 1);
                self.frontier.push_back((n, x + 1));
            }
        }
    }
}

