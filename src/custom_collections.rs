use std::collections::LinkedList;


pub struct Stack<T>(LinkedList<T>);


impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack(LinkedList::new())
    }

    pub fn push(&mut self, item: T) {
        self.0.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.back()
    }
}


pub struct Queue<T>(LinkedList<T>);


impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue(LinkedList::new())
    }

    pub fn enqueue(&mut self, item: T) {
        self.0.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.front()
    }
}
