use crate::ast::Ast;
use std::rc::Rc;
use ReOperator::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReOperator {
    Concat,
    Alter,
    Star,
    Left,
    Right,
}

impl ReOperator {
    pub fn priority(&self) -> i32 {
        match self {
            Left | Right => 0,
            Alter => 1,
            Concat => 2,
            Star => 3,
        }
    }
    pub fn eval(&self, ctx: &mut Vec<Ast<ReToken>>) {
        use ReOperator::*;
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
        ctx.push(Ast::new(ReToken::Operator(*self), children));
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReToken {
    Symbol(char),
    Operator(ReOperator),
}

impl ReToken {
    pub fn new(c: char) -> Self {
        match c {
            '*' => ReToken::Operator(Star),
            '|' => ReToken::Operator(Alter),
            '(' => ReToken::Operator(Left),
            ')' => ReToken::Operator(Right),
            _ => ReToken::Symbol(c),
        }
    }
    pub fn is_operator(&self) -> bool {
        !self.is_symbol()
    }
    pub fn is_symbol(&self) -> bool {
        if let ReToken::Symbol(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct Re {
    ast: Ast<ReToken>,
}

impl Re {
    pub fn new(pattern: &str) -> Self {
        let mut ops: Vec<ReOperator> = vec![];
        let mut asts: Vec<Ast<ReToken>> = vec![];
        // let `prev` be a `(` to push the first symbol into stack
        let mut prev = ReToken::Operator(Left);

        for c in pattern.chars() {
            // Construct a token from a char
            let mut c = ReToken::new(c);
            let mut temp = None;

            if c.is_symbol() {
                match prev {
                    ReToken::Operator(Alter) | ReToken::Operator(Left) => {
                        asts.push(Ast::new(c, vec![]));
                    }
                    _ => {
                        temp = Some(c);
                        c = ReToken::Operator(Concat);
                    }
                }
                prev = if temp.is_none() { c } else { temp.unwrap() }
            }

            if c.is_operator() {
                if let ReToken::Operator(func) = c {
                    match func {
                        Left => ops.push(func),
                        _ => {
                            while !ops.is_empty()
                                && func.priority() <= ops.last().unwrap().priority()
                            {
                                let func = ops.pop().unwrap();
                                if let ReOperator::Left = func {
                                    break;
                                }
                                func.eval(&mut asts);
                            }
                            if func != ReOperator::Right {
                                ops.push(func);
                            }
                            if func == ReOperator::Concat {
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
