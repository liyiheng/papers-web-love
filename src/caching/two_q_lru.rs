#![allow(dead_code)]
#![deny(missing_docs)]
use crate::common::list::{List, NodePtr};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

// KeyPosition represents a key in the A1 queue or the Am queue
#[derive(Eq, PartialEq, Copy, Clone)]
enum KeyPosition {
    A1,
    Am,
}
// Value contains data and extra info for 2Q
struct Value<K, V> {
    pos: KeyPosition,
    node: NodePtr<Rc<K>>,
    data: V,
}
impl<K, V> Value<K, V> {
    #[inline]
    fn at_am(&self) -> bool {
        self.pos == KeyPosition::Am
    }
}

/// Simplified 2Q
/// if p is on the Am queue
/// then
///      put p on the front of the Am queue
///      /* Am is managed as an LRU queue*/
/// else if p is on the A1 queue
/// then
///      remove p from the A1 queue
///      put p on the front of the Am queue
/// else /* first access we know about concerning p */
/// /* find a free page slot for p */
///      if there are free page slots available
///      then
///          put p in a free page slot
///      else if A1’s size is above a (tunable) threshold
///          delete from the tail of A1
///          put p in the freed page slot
///      else
///          delete from the tail of Am
///          put p in the freed page slot
///      end if
///      put p on the front of the A1 queue
/// end if
pub struct SimplifiedTwoQ<K: Eq + Hash, V> {
    lru: List<Rc<K>>,
    fifo: List<Rc<K>>,
    fifo_cap: usize,
    cap: usize,
    entries: HashMap<Rc<K>, Value<K, V>>,
}

impl<K: Eq + Hash, V> Drop for SimplifiedTwoQ<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Eq + Hash, V> SimplifiedTwoQ<K, V> {
    /// Create a new simplified 2Q with capacity.
    pub fn with_capacity(cap: usize) -> SimplifiedTwoQ<K, V> {
        let fifo_cap = if cap < 3 { cap } else { cap / 3 };
        Self::with_threshold(cap, fifo_cap)
    }

    /// Create a new simplified 2Q with capacity and A1 threshold.
    pub fn with_threshold(cap: usize, a1_threshold: usize) -> SimplifiedTwoQ<K, V> {
        SimplifiedTwoQ {
            lru: List::default(),
            fifo: List::default(),
            fifo_cap: a1_threshold,
            cap,
            entries: HashMap::new(),
        }
    }

    /// Remove all data in the cache.
    pub fn clear(&mut self) {
        while self.lru.pop_back().is_some() {}
        while self.fifo.pop_back().is_some() {}
        for v in self.entries.values_mut() {
            unsafe {
                std::ptr::drop_in_place(v.node.unwrap().as_ptr());
            }
        }
        self.entries.clear();
    }

    /// Get value with key.
    pub fn get(&mut self, k: &K) -> Option<&V> {
        if !self.entries.contains_key(k) {
            return None;
        }
        self.update(k);
        self.entries.get(k).map(|v| &v.data)
    }

    fn update(&mut self, k: &K) {
        let v = self.entries.get_mut(k).unwrap();
        if v.at_am() {
            unsafe {
                self.lru.remove_node(v.node);
                self.lru.push_front_node(v.node);
            }
        } else {
            unsafe {
                self.fifo.remove_node(v.node);
                self.lru.push_front_node(v.node);
            }
            v.pos = KeyPosition::Am;
        }
    }

    /// Insert K-V pair to the cache.
    pub fn insert(&mut self, k: K, v: V) {
        let k = Rc::new(k);
        if let Some(entry) = self.entries.get_mut(&k) {
            entry.data = v;
            self.update(&k);
        } else {
            // Eviction
            if self.entries.len() < self.cap {
            } else if self.fifo.len >= self.fifo_cap {
                let p = self.fifo.pop_back();
                unsafe {
                    self.entries.remove(&p.unwrap().as_ref().value);
                    std::ptr::drop_in_place(p.unwrap().as_ptr());
                }
            } else {
                let p = self.lru.pop_back();
                unsafe {
                    self.entries.remove(&p.unwrap().as_ref().value);
                    std::ptr::drop_in_place(p.unwrap().as_ptr());
                }
            }
            self.fifo.push_front(k.clone());
            self.entries.insert(
                k.clone(),
                Value {
                    pos: KeyPosition::A1,
                    data: v,
                    node: self.fifo.head,
                },
            );
        }
    }
}
