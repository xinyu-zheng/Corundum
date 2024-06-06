use std::fmt::Display;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::{Cell, RefCell};

const BUCKETS_MAX: usize = 16;

type VBucket<K> = Vec<RefCell<(K, usize)>>;

pub struct VHashMap<K, V> {
    buckets: Vec<RefCell<VBucket<K>>>,
    values: Vec<Cell<V>>,
}

impl<K, V: Copy> VHashMap<K, V> {
    pub fn foreach<F: FnMut(&K, V) -> ()>(&self, mut f: F) {
        for i in 0..BUCKETS_MAX {
            for e in &*self.buckets[i].borrow() {
                let e = e.borrow();
                f(&e.0, self.values[e.1].get());
            }
        }
    }
}

impl<K, V: Clone> VHashMap<K, V>
where
    K: PartialEq + Hash,
    V: Copy,
{
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(BUCKETS_MAX);
        for _ in 0..BUCKETS_MAX {
            buckets.push(RefCell::new(Vec::new()))
        }
        Self {
            buckets,
            values: Vec::new(),
        }
    }

    pub fn get(&self, key: K) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = (hasher.finish() as usize) % BUCKETS_MAX;

        for e in &*self.buckets[index].borrow() {
            let e = e.borrow();
            if e.0 == key {
                return Some(self.values[e.1].get());
            }
        }
        None
    }

    pub fn put(&mut self, key: K, val: V) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = (hasher.finish() as usize) % BUCKETS_MAX;
        let mut bucket = self.buckets[index].borrow_mut();

        for e in &*bucket {
            let e = e.borrow();
            if e.0 == key {
                self.values[e.1].set(val);
                return;
            }
        }

        self.values.push(Cell::new(val));
        bucket.push(RefCell::new((key, self.values.len() - 1)));
    }

    pub fn update_with<F: FnOnce(V) -> V>(&mut self, key: &K, f: F)
    where
        V: Default,
        K: Clone,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = (hasher.finish() as usize) % BUCKETS_MAX;
        let mut bucket = self.buckets[index].borrow_mut();

        for e in &*bucket {
            let e = e.borrow();
            if e.0 == *key {
                self.values[e.1].set(f(self.values[e.1].get()));
                return;
            }
        }

        self.values.push(Cell::new(f(V::default())));
        bucket.push(
            RefCell::new((key.clone(), self.values.len() - 1))
        );
    }

    pub fn clear(&mut self) {
        for i in 0..BUCKETS_MAX {
            self.buckets[i].borrow_mut().clear();
        }
        self.values.clear();
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..BUCKETS_MAX {
            if !self.buckets[i].borrow().is_empty() {
                return false;
            }
        }
        true
    }
}

impl<K: Display, V: Display + Copy> Display for VHashMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut vec = vec![];
        self.foreach(|word, freq| {
            vec.push((word.to_string(), freq.clone()));
        });
        vec.sort_by(|x, y| x.0.cmp(&y.0));
        for (word, freq) in vec {
            writeln!(f, "{:>32}: {}", word, freq)?;
        }
        Ok(())
    }
}