use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug)]
pub struct Vm<I: Hash + Eq> {
    start: usize,
    accept_states: HashSet<usize>,
    transitions: HashMap<(usize, I), usize>,
}

impl<I: Hash + Eq> Vm<I> {
    pub fn new(
        start: usize,
        accept_states: HashSet<usize>,
        transitions: HashMap<(usize, I), usize>,
    ) -> Self {
        Self {
            start,
            accept_states,
            transitions,
        }
    }
}

impl Vm<char> {
    fn jmp_table(&self) -> HashMap<usize, Vec<(char, usize)>> {
        let mut map: HashMap<usize, Vec<(char, usize)>> = HashMap::new();
        for rule in self.transitions.iter() {
            let (left, target) = rule;
            let (state, input) = left;
            map.entry(*state)
                .or_default()
                .push((input.clone(), *target));
        }
        map
    }

    fn switch_statement(&self) -> String {
        let jmp_table = self.jmp_table();
        jmp_table
            .into_iter()
            .map(|(header, branchs)| {
                let header = format!("\t\tcase {}: \n", header);
                let branchs = branchs
                    .into_iter()
                    .map(|(input, target)| {
                        format!(
                            "\t\t\tif(c == '{}') {{ state = {}; break; }}\n",
                            input, target
                        )
                    })
                    .fold(String::new(), |acc, ref string| acc + string);
                header + &branchs + "\t\t\tgoto error;\n"
            })
            .fold(String::new(), |acc, ref string| acc + string)
    }

    fn accept_statement(&self) -> String {
        self.accept_states
            .iter()
            .map(|state| format!("state == {}", state))
            .fold(String::new(), |acc, sub| {
                if acc.is_empty() {
                    sub
                } else {
                    acc + " || " + &sub
                }
            })
    }

    pub fn compile(self) -> String {
        let jmp_table = self.switch_statement();
        let program = format!(
            "
#include<stdio.h>
int main() {{
\tchar s[32];
\tscanf(\"%s\", s);
\tint state = {};
\tfor(int i = 0; s[i]!='\\0'; ++i) {{
\t\tchar c = s[i];
\t\tswitch(state) {{
{}
\t\tdefault:
\t\terror:
\t\t\tprintf(\"reject\\n\");
\t\t\treturn 0;
\t\t}}
\t}}
\tif({}) 
\t\tprintf(\"accept\\n\");
\telse
\t\tprintf(\"reject\\n\");
\treturn 0;
}}",
            self.start,
            jmp_table,
            self.accept_statement()
        );
        program
    }
}

#[test]
fn it_works() {
    use crate::dfa::*;
    use maplit::{hashmap, hashset};
    let start = 'a';
    let accept_states = hashset! {'c'};
    let transitions = hashmap! {
        ('a',Transition::new('a')) => 'b',
        ('a',Transition::new('b')) => 'c',
        ('b',Transition::new('b')) => 'c',
        ('c',Transition::new('c')) => 'b',
    };
    let dfa = Dfa::new(start, accept_states, transitions);
    let vm = Vm::from(dfa);
    println!("{}", vm.compile());
}
