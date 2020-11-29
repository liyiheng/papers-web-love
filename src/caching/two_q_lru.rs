#![allow(dead_code)]
#![deny(missing_docs)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ptr::NonNull;
use std::rc::Rc;

pub(crate) type NodePtr<T> = Option<NonNull<Node<T>>>;

#[derive(Debug)]
pub(crate) struct Node<T> {
    pub(crate) prev: NodePtr<T>,
    pub(crate) next: NodePtr<T>,
    pub(crate) value: T,
}
#[test]
fn test_list() {
    unsafe {
        let mut list = List::default();
        for i in (0..10).rev() {
            list.push_front(i);
        }
        for i in 10..20 {
            list.push_back(i);
        }
        for i in 0..20 {
            assert_eq!(i, list.get_node_at(i).unwrap().as_ref().value);
            assert_eq!(i, list.get_node_rev(19 - i).unwrap().as_ref().value);
        }
        for i in 0..19 {
            assert_eq!(list.len, 20 - i);
            if i % 2 == 0 {
                list.remove_node(list.head);
            } else {
                list.remove_node(list.tail);
            }
        }
        assert_eq!(list.len, 1);
        assert_eq!(list.tail, list.head);
        list.remove_node(list.tail);

        assert_eq!(list.len, 0);
        assert!(list.head.is_none() && list.tail.is_none());
    }
}

impl<T> Node<T> {
    pub(crate) fn new_ptr(t: T) -> NodePtr<T> {
        NonNull::new(Box::leak(Box::new(Node {
            value: t,
            prev: None,
            next: None,
        })))
    }
}

pub(crate) struct List<T> {
    pub(crate) len: usize,
    pub(crate) head: NodePtr<T>,
    pub(crate) tail: NodePtr<T>,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List {
            len: 0,
            head: None,
            tail: None,
        }
    }
}

impl<T> List<T> {
    // Node must in the list.
    pub(crate) unsafe fn remove_node(&mut self, node: Option<NonNull<Node<T>>>) {
        if self.head == node {
            self.pop_front();
        } else if self.tail == node {
            self.pop_back();
        } else {
            self.len -= 1;
            node.unwrap().as_ref().next.unwrap().as_mut().prev = node.unwrap().as_ref().prev;
            node.unwrap().as_ref().prev.unwrap().as_mut().next = node.unwrap().as_ref().next;
        }
    }

    pub(crate) fn push_front(&mut self, t: T) {
        self.push_front_node(Node::new_ptr(t));
    }

    pub(crate) fn push_back(&mut self, t: T) {
        self.push_back_node(Node::new_ptr(t));
    }

    pub(crate) fn push_back_node(&mut self, node: NodePtr<T>) {
        self.len += 1;
        unsafe {
            node.unwrap().as_mut().next = None;
            if let Some(mut tl) = self.tail {
                tl.as_mut().next = node;
                node.unwrap().as_mut().prev = self.tail;
                self.tail = node;
            } else {
                self.head = node;
                self.tail = node;
            }
        }
    }
    pub(crate) fn push_front_node(&mut self, node: NodePtr<T>) {
        self.len += 1;
        unsafe {
            node.unwrap().as_mut().prev = None;
            if let Some(mut hd) = self.head {
                hd.as_mut().prev = node;
                node.unwrap().as_mut().next = self.head;
                self.head = node;
            } else {
                self.head = node;
                self.tail = node;
            }
        }
    }

    pub(crate) fn pop_front(&mut self) -> NodePtr<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let hd = self.head;
        assert!(hd.is_some());
        unsafe {
            self.head = hd.unwrap().as_ref().next;
            if let Some(mut h) = self.head {
                h.as_mut().prev = None;
            }
        }
        if hd == self.tail {
            self.tail = None;
        }
        hd
    }

    pub(crate) fn pop_back(&mut self) -> NodePtr<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let tl = self.tail;
        unsafe {
            self.tail = tl.unwrap().as_ref().prev;
            if let Some(mut h) = self.tail {
                h.as_mut().next = None;
            }
        }
        if tl == self.head {
            self.head = None;
        }
        tl
    }

    fn get_node_at(&self, i: usize) -> NodePtr<T> {
        let mut n = self.head;
        for _ in 0..i {
            unsafe {
                n = n.unwrap().as_ref().next;
            }
        }
        n
    }

    fn get_node_rev(&self, i: usize) -> NodePtr<T> {
        let mut n = self.tail;
        for _ in 0..i {
            unsafe {
                n = n.unwrap().as_ref().prev;
            }
        }
        n
    }
}

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
