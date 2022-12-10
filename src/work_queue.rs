use std::collections::VecDeque;
use std::sync::{ Arc, Condvar, Mutex };

pub fn make_queue<T: Send>(size: usize) -> (Sender<T>, Receiver<T>) {
    let store = Arc::new(Mutex::new(VecDeque::with_capacity(size)));
    let cv = Arc::new(Condvar::new());

    (Sender {
        cv: Arc::clone(&cv),
        store: Arc::clone(&store),
    },
    Receiver {
        cv,
        store,
    })
}

pub struct Sender<T: Send> {
    cv: Arc<Condvar>,
    store: Arc<Mutex<VecDeque<T>>>,
}

pub struct Receiver<T: Send> {
    cv: Arc<Condvar>,
    store: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send> Sender<T> {
    pub fn dispatch(&mut self, data: T) {
        let mut store = self.store.lock().unwrap();
        store.push_front(data);
        drop(store);

        self.cv.notify_one();
    }
}

impl<T: Send> Receiver<T> {
    pub fn find_work(&mut self) -> T {
        let lock = self.store.lock().unwrap();
        let mut store = self.cv.wait_while(lock,
            |store: &mut VecDeque<T>| { store.len() == 0 }).unwrap();
        let work = store.pop_back().unwrap();
        drop(store);

        work
    }
}

impl<T: Send> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.find_work())
    }
}

impl<T: Send> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            cv: Arc::clone(&self.cv),
            store: Arc::clone(&self.store),
        }
    }
}
