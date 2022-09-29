use std::collections::VecDeque;

pub struct WorkQueue<T> {
    queue: VecDque<T>,
}

impl<T> WorkQueue<T> {
    pub fn new<T, F>(thread_count: usize, handler: F)
        where F: FnMut() -> bool
    {
        WorkQueue<T> {
            queue: VecQueue::new(),
        }
    }
}

