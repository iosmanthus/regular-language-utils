use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Ast<T> {
    token: T,
    children: Vec<Rc<Ast<T>>>,
}

impl<T> Ast<T> {
    pub fn new(token: T, children: Vec<Rc<Ast<T>>>) -> Self {
        Ast { token, children }
    }
}
