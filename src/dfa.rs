use crate::automatan::Trace;
use crate::vm::Vm;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Transition<I>(I);

impl<I> Transition<I> {
    pub fn new(c: I) -> Self {
        Transition(c)
    }
    pub fn inner_symbol(self) -> I {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SetState<S: Hash + Eq>(HashSet<S>);

impl<S: Hash + Eq> SetState<S> {
    pub fn new(set: HashSet<S>) -> Self {
        SetState(set)
    }
}

impl<S: Hash + Eq + PartialOrd> PartialEq for SetState<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0) && other.0.is_subset(&self.0)
    }
}

impl<S: Hash + Eq + PartialOrd> Eq for SetState<S> {}

impl<S: Hash + Eq + Ord> Hash for SetState<S> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let mut a: Vec<&S> = self.0.iter().collect();
        a.sort();
        for s in a.iter() {
            s.hash(state);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dfa<S, I>
where
    S: Hash + Eq,
    I: Hash + Eq,
{
    start: S,
    accept_states: HashSet<S>,
    transitions: HashMap<(S, Transition<I>), S>,
}

impl<S, I> Dfa<S, I>
where
    S: Hash + Eq,
    I: Hash + Eq,
{
    pub fn new(
        start: S,
        accept_states: HashSet<S>,
        transitions: HashMap<(S, Transition<I>), S>,
    ) -> Self {
        Self {
            start,
            accept_states,
            transitions,
        }
    }

    pub fn add_transition(&mut self, transition: ((S, Transition<I>), S)) {
        self.transitions.insert(transition.0, transition.1);
    }
}

impl<S, I> Dfa<S, I>
where
    S: Hash + Eq + Clone,
    I: Hash + Eq + Clone,
{
    pub fn run(&self, input: &[I]) -> Trace<S> {
        let mut state = self.start.clone();
        let mut trace = vec![];
        for symbol in input {
            trace.push(state.clone());
            let next = self
                .transitions
                .get(&(state.clone(), Transition::new(symbol.clone())))
                .cloned();
            if next.is_none() {
                return Trace::new(false, trace);
            }
            state = next.unwrap();
        }
        let accept = self.accept_states.contains(&state);
        trace.push(state);
        Trace::new(accept, trace)
    }
}

impl<S, I> From<Dfa<S, I>> for Vm<I>
where
    S: Hash + Eq + Clone,
    I: Hash + Eq + Clone,
{
    fn from(dfa: Dfa<S, I>) -> Self {
        let consume_id = |id: &mut usize| {
            let old_id = *id;
            *id += 1;
            old_id
        };

        let mut vm_transitions = HashMap::new();
        let mut state_to_usize = HashMap::new();
        let mut id = 0;
        for rule in dfa.transitions {
            let (left, right) = rule;
            let (state, input) = left;

            let left_id = *state_to_usize.entry(state).or_insert(consume_id(&mut id));
            let right_id = *state_to_usize.entry(right).or_insert(consume_id(&mut id));
            vm_transitions.insert((left_id, input.clone().inner_symbol()), right_id);
        }

        let start = *state_to_usize.get(&dfa.start).unwrap();
        let accept_states = dfa
            .accept_states
            .into_iter()
            .map(|state| *state_to_usize.entry(state).or_insert(consume_id(&mut id)))
            .collect::<HashSet<_>>();
        Vm::new(start, accept_states, vm_transitions)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use maplit::{hashmap, hashset};
    #[test]
    fn test_dfa_run() {
        let start = 0;
        let accept_states = hashset! {2};
        let transitions = hashmap! {
            (0,Transition::new('a')) => 1,
            (1,Transition::new('b')) => 2,
            (2,Transition::new('c')) => 1,
        };
        let dfa = Dfa::new(start, accept_states, transitions);
        assert_eq!(vec![0, 1, 2, 1, 2], dfa.run(&['a', 'b', 'c', 'b']).trace());
    }
    #[test]
    fn test_dfa_to_vm() {
        let start = 'a';
        let accept_states = hashset! {'c'};
        let transitions = hashmap! {
            ('a',Transition::new('a')) => 'b',
            ('a',Transition::new('b')) => 'c',
            ('b',Transition::new('b')) => 'c',
            ('c',Transition::new('c')) => 'b',
        };
        let dfa = Dfa::new(start, accept_states, transitions);
        dbg!(Vm::from(dfa));
    }
}
