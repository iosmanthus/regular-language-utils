use std::collections::HashSet;
use std::hash::Hash;

pub fn power_set<S>(mut set: HashSet<S>) -> Vec<HashSet<S>>
where
    S: Hash + Eq + Clone,
{
    if set.is_empty() {
        vec![HashSet::new()]
    } else {
        let element = set.iter().last().cloned().unwrap();
        set.remove(&element);
        let mut rest = power_set(set);

        let mut included = rest.clone();
        included.iter_mut().for_each(|set| {
            set.insert(element.clone());
        });
        rest.extend(included);

        rest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;
    use std::iter::FromIterator;
    #[test]
    fn test_power_set() {
        let power_set = power_set(hashset! {1,2,3});
        let power_set: HashSet<_> = HashSet::from_iter(power_set.into_iter().map(|set| {
            let mut vec = Vec::from_iter(set.into_iter());
            vec.sort();
            vec
        }));
        assert_eq!(
            hashset! {
                vec![],
                vec![1],
                vec![2],
                vec![3],
                vec![1,2],
                vec![1,3],
                vec![2,3],
                vec![1,2,3],
            },
            power_set
        );
    }
}
