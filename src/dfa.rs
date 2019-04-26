use std::collections::HashMap;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Transition<I> {
    Symbol(I),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dfa<S, I>
where
    S: Hash + Eq,
    I: Hash + Eq,
{
    start: S,
    accept_state: S,
    transitions: HashMap<(S, Transition<I>), S>,
}
