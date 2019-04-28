use maplit::hashset;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use crate::automatan::Trace;
use crate::dfa::{self, Dfa, SetState};
use crate::utils;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Transition<I> {
    Epsilon,
    Symbol(I),
}

impl<I> Transition<I> {
    pub fn inner_symbol(self) -> Option<I> {
        if let Transition::Symbol(c) = self {
            Some(c)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn add_transition(&mut self, transitions: ((S, Transition<I>), HashSet<S>)) {
        let entry = self.transitions.entry(transitions.0).or_default();
        entry.extend(transitions.1);
    }

    pub fn get_transition<'a>(&'a self, input: &(S, Transition<I>)) -> Option<&'a HashSet<S>> {
        self.transitions.get(input)
    }
}

impl<S, I> Nfa<S, I>
where
    S: Hash + Eq + Clone,
    I: Hash + Eq + Clone,
{
    pub fn concat(mut self, other: Self) -> Self {
        use std::mem;
        for state in mem::replace(&mut self.accept_states, other.accept_states) {
            self.add_transition(((state, Transition::Epsilon), hashset! {other.start.clone()}));
        }
        self.transitions.extend(other.transitions);
        self
    }

    pub fn union(mut self, mut other: Self, start: S) -> Self {
        use std::mem;
        // merge two transitions function sets.
        self.transitions
            .extend(mem::replace(&mut other.transitions, HashMap::new()));

        self.add_transition((
            (start.clone(), Transition::Epsilon),
            hashset! {self.start.clone(),other.start.clone()},
        ));

        self.accept_states.extend(other.accept_states);
        self.start = start;
        self
    }

    pub fn star(mut self, start: S, accept: S) -> Self {
        use std::mem;
        self.add_transition((
            (start.clone(), Transition::Epsilon),
            hashset! {accept.clone(),self.start.clone()},
        ));

        for state in mem::replace(&mut self.accept_states, HashSet::new()) {
            self.add_transition((
                (state.clone(), Transition::Epsilon),
                hashset! {start.clone()},
            ));
        }

        self.accept_states.insert(accept);
        self.start = start;
        self
    }

    /// extend a state set with epsilon edge
    pub fn extend_set(nfa: &Nfa<S, I>, set: &HashSet<S>) -> HashSet<S> {
        let mut extend = set.clone();
        let mut queue = VecDeque::new();
        queue.extend(extend.iter().cloned().collect::<VecDeque<_>>());
        while !queue.is_empty() {
            let state = queue.pop_front().unwrap();
            let detected = nfa
                .get_transition(&(state, Transition::Epsilon))
                .cloned()
                .unwrap_or_default();
            for state in detected {
                if extend.insert(state.clone()) {
                    queue.push_back(state);
                }
            }
        }
        extend
    }

    pub fn run(&self, input: &[I]) -> Trace<HashSet<S>> {
        let mut trace = vec![];
        let mut set = Nfa::extend_set(self, &hashset! {self.start.clone()});

        for symbol in input {
            trace.push(set.clone());
            set = Nfa::extend_set(
                self,
                &set.into_iter()
                    .map(|state| {
                        self.get_transition(&(state, Transition::Symbol(symbol.clone())))
                            .cloned()
                            .unwrap_or_default()
                    })
                    .flatten()
                    .collect::<HashSet<_>>(),
            );
        }
        let accept = set.iter().any(|state| self.accept_states.contains(&state));
        trace.push(set);
        Trace::new(accept, trace)
    }
}

impl<S, I> From<Nfa<S, I>> for Dfa<SetState<S>, I>
where
    S: Hash + Eq + Ord + Clone,
    I: Hash + Eq + Clone,
{
    fn from(nfa: Nfa<S, I>) -> Dfa<SetState<S>, I> {
        let mut valid_input = HashMap::<_, HashSet<_>>::new();
        let mut state_set = HashSet::new();
        for (input, output) in nfa.transitions.iter() {
            let (k, v) = input;
            valid_input.entry(k).or_default().insert(v.clone());
            state_set.insert(k.clone());
            state_set.extend(output.clone());
        }

        let power_set = utils::power_set(state_set);

        let start = hashset! {nfa.start.clone()};
        let accept_states = power_set
            .iter()
            .cloned()
            .filter(|subset| !subset.is_disjoint(&nfa.accept_states))
            .map(|subset| SetState::new(subset.clone()))
            .collect::<HashSet<_>>();

        let mut transitions = HashMap::new();
        for set in power_set {
            let extend = Nfa::extend_set(&nfa, &set);
            let valid_input = extend
                .iter()
                .map(|state| valid_input.get(state).cloned().unwrap_or_default())
                .flatten()
                .collect::<HashSet<_>>();
            for symbol in valid_input
                .into_iter()
                .filter(|input| *input != Transition::Epsilon)
            {
                let next = Nfa::extend_set(
                    &nfa,
                    &extend
                        .iter()
                        .map(|state| {
                            nfa.get_transition(&(state.clone(), symbol.clone()))
                                .cloned()
                                .unwrap_or_default()
                        })
                        .flatten()
                        .collect(),
                );

                transitions.insert(
                    (
                        SetState::new(set.clone()),
                        dfa::Transition::new(symbol.inner_symbol().unwrap()),
                    ),
                    SetState::new(next),
                );
            }
        }
        Dfa::new(
            SetState::new(Nfa::extend_set(&nfa, &start)),
            accept_states,
            transitions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    #[test]
    fn test_nfa_run() {
        let start = 0;
        let accept_states = hashset! {3,5};
        let transitions = hashmap! {
            (0,Transition::Epsilon) => hashset!{1,4},
            (1,Transition::Symbol('a')) => hashset!{2},
            (2,Transition::Epsilon) => hashset!{3},
            (4,Transition::Symbol('b')) => hashset!{5},
        };
        let nfa = Nfa::new(start, accept_states, transitions);

        assert_eq!(
            vec![hashset! {0,1,4}, hashset! {2,3}],
            nfa.run(&['a']).trace()
        );
        assert_eq!(true, nfa.run(&['a']).accept());
        assert_eq!(true, nfa.run(&['b']).accept());
        assert_eq!(false, nfa.run(&['a', 'b']).accept());
    }

    #[test]
    fn test_nfa_concat() {
        let start = 1;
        let accept_states = hashset! {2};
        let transitions = hashmap! {
            (1,Transition::Symbol('a')) => hashset!{2},
        };
        let nfa_a = Nfa::new(start, accept_states, transitions);

        let start = 3;
        let accept_states = hashset! {4};
        let transitions = hashmap! {
            (3,Transition::Symbol('b')) => hashset!{4},
        };
        let nfa_b = Nfa::new(start, accept_states, transitions);

        let start = 1;
        let accept_states = hashset! {4};
        let transitions = hashmap! {
            (1,Transition::Symbol('a')) => hashset!{2},
            (2,Transition::Epsilon) => hashset!{3},
            (3,Transition::Symbol('b')) => hashset!{4},
        };

        assert_eq!(
            Nfa::new(start, accept_states, transitions),
            nfa_a.concat(nfa_b)
        );
    }

    #[test]
    fn test_nfa_alter() {
        let start = 1;
        let accept_states = hashset! {2};
        let transitions = hashmap! {
            (1,Transition::Symbol('a')) => hashset!{2},
        };
        let nfa_a = Nfa::new(start, accept_states, transitions);

        let start = 3;
        let accept_states = hashset! {4};
        let transitions = hashmap! {
            (3,Transition::Symbol('b')) => hashset!{4},
        };
        let nfa_b = Nfa::new(start, accept_states, transitions);

        let start = 0;
        let accept_states = hashset! {2,4};
        let transitions = hashmap! {
            (0,Transition::Epsilon) => hashset! {1,3},
            (1,Transition::Symbol('a')) => hashset!{2},
            (3,Transition::Symbol('b')) => hashset!{4},
        };

        assert_eq!(
            Nfa::new(start, accept_states, transitions),
            nfa_a.union(nfa_b, 0)
        );
    }

    #[test]
    fn test_nfa_star() {
        let start = 1;
        let accept_states = hashset! {2};
        let transitions = hashmap! {
            (1,Transition::Symbol('a')) => hashset!{2},
        };
        let nfa_a = Nfa::new(start, accept_states, transitions);

        let start = 0;
        let accept_states = hashset! {3};
        let transitions = hashmap! {
            (0,Transition::Epsilon) => hashset!{1,3},
            (1,Transition::Symbol('a')) => hashset!{2},
            (2,Transition::Epsilon) => hashset!{0}
        };

        assert_eq!(
            Nfa::new(start, accept_states, transitions),
            nfa_a.star(0, 3)
        );
    }

    #[test]
    fn test_nfa_to_dfa() {
        let start = 0;
        let accept_states = hashset! {2};
        let transitions = hashmap! {
            (0, Transition::Epsilon) => hashset! {1,3},
            (1, Transition::Symbol('a')) => hashset! {2},
        };
        let nfa = Nfa::new(start, accept_states, transitions);
        dbg!(Dfa::from(nfa));
    }

}
