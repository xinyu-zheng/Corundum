use crate::map::Map;
use corundum::default::*;

use std::{cell::UnsafeCell, collections::BTreeMap};

type P = Allocator;

pub struct RTree<K, V> {
    btree: UnsafeCell<BTreeMap<K, V>>,
}

impl<K, V: Copy> Map<K, V> for RTree<K, V>
where
    K: std::cmp::Ord,
{
    fn clear(&self) {
        unsafe { (*self.btree.get()).clear(); }
    }
    fn insert(&self, key: K, val: V) {
        unsafe { (*self.btree.get()).insert(key, val); }
    }
    fn remove(&self, key: K) {
        unsafe { (*self.btree.get()).remove(&key); }
    }
    fn is_empty(&self) -> bool {
        unsafe { (*self.btree.get()).is_empty() }
    }
    fn foreach<F: Copy + Fn(&K, &V) -> bool>(&self, f: F) -> bool {
        unsafe {
                for (k, v) in &*self.btree.get() {
                f(k, v);
            }
            true
        }
    }
    fn lookup(&self, key: K) -> bool {
        unsafe { (*self.btree.get()).get(&key).is_some() }
    }
}

impl<K: std::cmp::Ord, V> Default for RTree<K, V> {
    fn default() -> Self {
        Self {
            btree: UnsafeCell::new(BTreeMap::new()),
        }
    }
}
