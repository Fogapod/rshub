use core::borrow::Borrow;

use std::cmp::Ord;
use std::collections::{btree_set::Iter, BTreeSet, HashMap};
use std::hash::Hash;

pub struct ValueSortedMap<K, V> {
    map: HashMap<K, V>,
    set: BTreeSet<V>,
}

impl<K: Eq + Hash, V: Ord + Clone> ValueSortedMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            set: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.map.insert(k, v.clone());
        self.set.replace(v);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.map.get(k)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&V) -> bool,
    {
        self.map.retain(|_, v| f(v));
        self.set.retain(f)
    }

    pub fn remove_value<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        V: Borrow<Q> + Ord + PartialEq<Q>,
        Q: Ord,
    {
        if self.set.remove(value) {
            self.map.retain(|_, v| v == value);

            true
        } else {
            false
        }
    }
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn iter(&self) -> Iter<'_, V> {
        self.set.iter()
    }
}
