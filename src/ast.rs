use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Ast<T> {
    token: T,
    children: Option<Vec<Rc<Ast<T>>>>,
}

impl<T> Ast<T> {
    pub fn new(token: T, children: Option<Vec<Rc<Ast<T>>>>) -> Self {
        Ast { token, children }
    }

    pub fn token(&self) -> &T {
        &self.token
    }

    pub fn children(&self) -> Option<&Vec<Rc<Ast<T>>>> {
        self.children.as_ref()
    }
}
