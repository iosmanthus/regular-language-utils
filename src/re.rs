use crate::ast::Ast;
use crate::nfa::{Nfa, Transition};
use maplit::{hashmap, hashset};
use std::rc::Rc;
use ReOperator::*;
use ReToken::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReOperator {
    Concat,
    Alter,
    Star,
    Left,
    Right,
}

impl ReOperator {
    pub fn priority(self) -> i32 {
        match self {
            Left | Right => 0,
            Alter => 1,
            Concat => 2,
            Star => 3,
        }
    }
    pub fn eval(self, ctx: &mut Vec<Ast<ReToken>>) {
        let pcnt = match self {
            Concat | Alter => 2,
            Star => 1,
            Left | Right => 0,
        };
        let mut children = vec![];
        for _ in 0..pcnt {
            children.push(Rc::new(ctx.pop().unwrap()));
        }
        children.reverse();
        let children = if !children.is_empty() {
            Some(children)
        } else {
            None
        };
        ctx.push(Ast::new(Operator(self), children));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReToken {
    Symbol(char),
    Operator(ReOperator),
}

impl ReToken {
    pub fn new(c: char) -> Self {
        match c {
            '*' => Operator(Star),
            '|' => Operator(Alter),
            '(' => Operator(Left),
            ')' => Operator(Right),
            _ => Symbol(c),
        }
    }
    pub fn is_operator(self) -> bool {
        !self.is_symbol()
    }
    pub fn is_symbol(self) -> bool {
        if let Symbol(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Re {
    ast: Ast<ReToken>,
}

impl Re {
    pub fn new(pattern: &str) -> Self {
        let mut ops: Vec<ReOperator> = vec![];
        let mut asts: Vec<Ast<ReToken>> = vec![];
        // let `prev` be a `(` to push the first symbol into stack
        let mut prev = Operator(Left);

        for c in pattern.chars() {
            // Construct a token from a char
            let mut c = ReToken::new(c);
            let mut temp = None;

            if c.is_symbol() {
                match prev {
                    Operator(Alter) | Operator(Left) => {
                        asts.push(Ast::new(c, None));
                    }
                    _ => {
                        temp = Some(c);
                        c = Operator(Concat);
                    }
                }
                prev = if temp.is_none() { c } else { temp.unwrap() }
            }

            if c.is_operator() {
                if let Operator(func) = c {
                    match func {
                        Left => ops.push(func),
                        _ => {
                            while !ops.is_empty()
                                && func.priority() <= ops.last().unwrap().priority()
                            {
                                let func = ops.pop().unwrap();
                                if let Left = func {
                                    break;
                                }
                                func.eval(&mut asts);
                            }
                            if func != Right {
                                ops.push(func);
                            }
                            if func == Concat {
                                asts.push(Ast::new(temp.unwrap(), None));
                            } else {
                                prev = c;
                            }
                        }
                    }
                }
            }
        }

        while !ops.is_empty() {
            let func = ops.pop().unwrap();
            func.eval(&mut asts);
        }
        Re {
            ast: asts[0].clone(),
        }
    }

    fn ast(&self) -> &Ast<ReToken> {
        &self.ast
    }
}

impl From<Re> for Nfa<usize, char> {
    fn from(re: Re) -> Self {
        use ReOperator::*;
        use ReToken::*;
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
                    let result = l.union(r, *id);
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
    fn test_re_parse() {
        let re = Re::new("(1*2)|3");
        let ast = Ast::new(
            Operator(Alter),
            Some(vec![
                Rc::new(Ast::new(
                    Operator(Concat),
                    Some(vec![
                        Rc::new(Ast::new(
                            Operator(Star),
                            Some(vec![Rc::new(Ast::new(Symbol('1'), None))]),
                        )),
                        Rc::new(Ast::new(Symbol('2'), None)),
                    ]),
                )),
                Rc::new(Ast::new(Symbol('3'), None)),
            ]),
        );
        assert_eq!(Re { ast }, re);
    }

    #[test]
    fn test_nfa_from_re() {
        use crate::re::Re;
        let start = 2;
        let accept_states = hashset! {7};
        let transitions = hashmap! {
            (0,Transition::Symbol('a')) => hashset!{1},
            (4,Transition::Symbol('b')) => hashset!{5},
            (6,Transition::Epsilon) => hashset!{4,7},
            (1,Transition::Epsilon) => hashset!{2},
            (5,Transition::Epsilon) => hashset!{6},
            (2,Transition::Epsilon) => hashset!{0,3},
            (3,Transition::Epsilon) => hashset!{6},
        };
        assert_eq!(
            Nfa::new(start, accept_states, transitions),
            Nfa::from(Re::new("a*b*"))
        );
    }
}
