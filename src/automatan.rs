#[derive(Debug, Clone)]
pub struct Trace<S> {
    accept: bool,
    trace: Vec<S>,
}

impl<S> Trace<S> {
    pub fn new(accept: bool, trace: Vec<S>) -> Self {
        Self { accept, trace }
    }
    pub fn push(&mut self, state: S) {
        self.trace.push(state);
    }
    pub fn accept(&self) -> bool {
        self.accept
    }
    pub fn trace(&self) -> &[S] {
        &self.trace
    }
}
