use crate::re::Re;
use maplit::{hashmap, hashset};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Transition<I> {
    Epsilon,
    Symbol(I),
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

    pub fn alter(mut self, mut other: Self, start: S) -> Self {
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

    pub fn run(&self, input: &[I]) -> bool {
        let mut set = hashset! {self.start.clone()};
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
                    if !extend.insert(state.clone()) {
                        queue.push_back(state);
                    }
                }
            }
            set = extend
                .into_iter()
                .map(|state| {
                    self.transitions
                        .get(&(state, Transition::Symbol(symbol.clone())))
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

impl From<Re> for Nfa<usize, char> {
    fn from(re: Re) -> Self {
        use crate::ast::Ast;
        use crate::re::{ReOperator, ReToken};
        use ReOperator::*;
        use ReToken::*;
        #[allow(dead_code)]
        fn from(ast: &Ast<ReToken>, id: &mut usize) -> Nfa<usize, char> {
            match ast.token() {
                &Symbol(a) => {
                    let result = Nfa::new(
                        *id,
                        hashset! {*id+1},
                        hashmap! {
                            (*id,Transition::Symbol(a)) => hashset! {*id+1}
                        },
                    );
                    // Consume two state id
                    *id += 2;
                    result
                }
                Operator(Concat) => {
                    let children = ast.children().unwrap();
                    let (l, r) = (from(&children[0], id), from(&children[1], id));
                    l.concat(r)
                }

                Operator(Alter) => {
                    let children = ast.children().unwrap();
                    let (l, r) = (from(&children[0], id), from(&children[1], id));
                    let result = l.alter(r, *id);
                    *id += 1;
                    result
                }

                Operator(Star) => {
                    let children = ast.children().unwrap();
                    let leaf = from(&children[0], id);
                    let result = leaf.star(*id, *id + 1);
                    *id += 2;
                    result
                }
                _ => unreachable!(),
            }
        }
        from(re.ast(), &mut 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            nfa_a.alter(nfa_b, 0)
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
    fn test_nfa_from_re() {
        use crate::re::Re;
        dbg!(Nfa::from(Re::new("a*b*")));
        dbg!(Nfa::from(Re::new("ab|c*")));
        dbg!(Nfa::from(Re::new("(a|b)*")));
    }
}
