use crate::map::Map;
use corundum::default::*;

use std::{cell::UnsafeCell, collections::BTreeMap};

type P = Allocator;

pub struct RTree<K, V> {
    btree: UnsafeCell<BTreeMap<K, V>>,
}

impl<K, V> RTree<K, V> {
    fn get(&self) -> &mut BTreeMap<K, V> {
        unsafe { &mut *self.btree.get() }
    }
}

impl<K, V: Copy> Map<K, V> for RTree<K, V>
where
    K: std::cmp::Ord,
{
    fn clear(&self) {
        self.get().clear();
    }
    fn insert(&self, key: K, val: V) {
        self.get().insert(key, val);
    }
    fn remove(&self, key: K) {
        self.get().remove(&key);
    }
    fn is_empty(&self) -> bool {
        self.get().is_empty()
    }
    fn foreach<F: Copy + Fn(&K, &V) -> bool>(&self, f: F) -> bool {
        for (k, v) in self.get() {
            f(k, v);
        }
        true
    }
    fn lookup(&self, key: K) -> bool {
        self.get().get(&key).is_some()
    }
}

impl<K: std::cmp::Ord, V> Default for RTree<K, V> {
    fn default() -> Self {
        Self {
            btree: UnsafeCell::new(BTreeMap::new()),
        }
    }
}
