use core::ops::Index;

use std::cmp::Ord;
use std::collections::{btree_set::Iter, BTreeMap, BTreeSet};

pub struct ValueSortedMap<K, V> {
    map: BTreeMap<K, V>,
    set: BTreeSet<V>,
}

impl<K: Ord, V: Ord + PartialEq<K> + Clone> ValueSortedMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            set: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // pop entry using key (dumb) cmp
        let removed = if let Some((removed_key, removed_value)) = self.map.remove_entry(&k) {
            // remove entry from set using dumb cmp to key
            self.set.retain(|i| i != &removed_key);
            Some(removed_value)
        } else {
            None
        };

        self.map.insert(k, v.clone());
        self.set.insert(v);

        removed
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

    pub fn remove(&mut self, k: &K) {
        self.map.remove(k).map(|v| self.set.remove(&v));
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn iter(&self) -> Iter<'_, V> {
        self.set.iter()
    }
}

impl<K, V> Index<usize> for ValueSortedMap<K, V> {
    type Output = V;

    // FIXME: this is O(n) !!!
    fn index(&self, i: usize) -> &V {
        self.set.iter().nth(i).expect("no entry found for index")
    }
}
