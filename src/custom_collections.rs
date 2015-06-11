use std::collections::LinkedList;
use std::iter::FromIterator;


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

    pub fn len(&self) -> usize {
        self.0.len()
    }
}


impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Stack<T> {
        let mut stack = Stack::new();
        for item in iterable.into_iter() {
            stack.push(item);
        }
        stack
    }
}


pub struct Queue<T>(pub LinkedList<T>);


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

    pub fn len(&self) -> usize {
        self.0.len()
    }
}


impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Queue<T> {
        let mut queue = Queue::new();
        for item in iterable.into_iter() {
            queue.enqueue(item);
        }
        queue
    }
}
