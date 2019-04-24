use maplit::hashset;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Transition<I> {
    Epsilon,
    Symbol(I),
}

#[derive(Clone, Debug)]
pub struct Nfa<S, I>
where
    S: Hash + Eq,
    I: Hash + Eq,
{
    start: S,
    accept_states: HashSet<S>,
    transitions: HashMap<(S, Transition<I>), HashSet<S>>,
}

impl<S, I> Nfa<S, I>
where
    S: Hash + Eq,
    I: Hash + Eq,
{
    pub fn new(
        start: S,
        accept_states: HashSet<S>,
        transitions: HashMap<(S, Transition<I>), HashSet<S>>,
    ) -> Self {
        Self {
            start,
            accept_states,
            transitions,
        }
    }
}

impl<S, I> Nfa<S, I>
where
    S: Hash + Eq + Copy,
    I: Hash + Eq + Copy,
{
    pub fn run(&self, input: &[I]) -> bool {
        let mut set = hashset! {self.start};
        for symbol in input {
            let mut extend = set.clone();
            let mut queue = VecDeque::new();
            queue.extend(extend.iter().cloned().collect::<VecDeque<_>>());
            while !queue.is_empty() {
                let state = queue.pop_front().unwrap();
                let detected = self
                    .transitions
                    .get(&(state, Transition::Epsilon))
                    .cloned()
                    .unwrap_or_default();
                for state in detected {
                    if !extend.insert(state) {
                        queue.push_back(state);
                    }
                }
            }
            set = extend
                .into_iter()
                .map(|state| {
                    self.transitions
                        .get(&(state, Transition::Symbol(*symbol)))
                        .cloned()
                        .unwrap_or_default()
                })
                .flatten()
                .collect::<HashSet<_>>();
        }
        set.into_iter()
            .any(|state| self.accept_states.contains(&state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    #[test]
    fn test_nfa_run() {
        let start = 1;
        let accept_states = hashset! {4, 5};
        let transitions = hashmap! {
            (1,Transition::Epsilon) => hashset!{2, 3, 6},
            (2,Transition::Symbol('a')) => hashset!{4},
            (3,Transition::Symbol('b')) => hashset!{5},
            (6,Transition::Symbol('c')) => hashset!{5},
        };
        let nfa = Nfa::new(start, accept_states, transitions);

        assert_eq!(true, nfa.run(&['a']));
        assert_eq!(true, nfa.run(&['b']));
        assert_eq!(true, nfa.run(&['c']));
        assert_eq!(false, nfa.run(&['a', 'b']));
    }
}
