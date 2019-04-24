use crate::ast::Ast;
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
                        asts.push(Ast::new(c, vec![]));
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
                                asts.push(Ast::new(temp.unwrap(), vec![]));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_re_parse() {
        let re = Re::new("(1*2)|3");
        let ast = Ast::new(
            Operator(Alter),
            vec![
                Rc::new(Ast::new(
                    Operator(Concat),
                    vec![
                        Rc::new(Ast::new(
                            Operator(Star),
                            vec![Rc::new(Ast::new(Symbol('1'), vec![]))],
                        )),
                        Rc::new(Ast::new(Symbol('2'), vec![])),
                    ],
                )),
                Rc::new(Ast::new(Symbol('3'), vec![])),
            ],
        );
        assert_eq!(Re { ast }, re);
    }
}
