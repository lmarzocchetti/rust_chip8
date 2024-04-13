#[derive(Debug)]
pub struct Stack<T> {
    #[allow(dead_code)]
    values: std::vec::Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { values: vec![] }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, val: T) {
        self.values.push(val)
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> T {
        self.values.pop().unwrap()
    }
}
