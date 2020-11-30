#![allow(dead_code)]
#![deny(missing_docs)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::Hash;
use std::ptr::NonNull;
use std::rc::Rc;

#[test]
fn test_list() {
    unsafe {
        let mut list = Link::new(123);
        for i in (0..10).rev() {
            list.push_front(i);
        }
        for i in 10..20 {
            list.push_back(i);
        }
        assert_eq!(list.len, 20);
        for i in 0..20 {
            assert_eq!(i, list.get_node_at(i).unwrap().as_ref().value);
            assert_eq!(i, list.get_node_rev(19 - i).unwrap().as_ref().value);
        }
        for i in 0..19 {
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

#[inline]
fn to_raw<T>(t: T) -> *mut T {
    Box::leak(Box::new(t))
}

#[derive(Debug)]
struct Node<T> {
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    list: Option<NonNull<Link<T>>>,
    value: T,
}

struct Link<T> {
    times: u64,
    len: usize,
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Link<T>>>,
    next: Option<NonNull<Link<T>>>,
}

impl<T> Debug for Link<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            writeln!(f, "Link {:?}", self as *const _)?;
            let mut cur = self.head;
            while let Some(c) = cur {
                write!(f, "{:?} -> ", c.as_ptr())?;
                if c.as_ref().next.is_none() {
                    break;
                }
                cur = c.as_ref().next;
            }
            writeln!(f, "NULL")
        }
    }
}

impl<T> Link<T> {
    fn new(times: u64) -> Link<T> {
        Link {
            times,
            len: 0,
            head: None,
            tail: None,
            prev: None,
            next: None,
        }
    }

    fn get_node_at<'a>(&'a self, i: usize) -> Option<NonNull<Node<T>>> {
        let mut n = self.head;
        for _ in 0..i {
            unsafe {
                n = n.unwrap().as_ref().next;
            }
        }
        n
    }

    fn get_node_rev<'a>(&'a self, i: usize) -> Option<NonNull<Node<T>>> {
        let mut n = self.tail;
        for _ in 0..i {
            unsafe {
                n = n.unwrap().as_ref().prev;
            }
        }
        n
    }

    fn push_back(&mut self, v: T) {
        self.len += 1;
        let node = NonNull::new(to_raw(Node {
            value: v,
            next: None,
            prev: self.tail,
            list: NonNull::new(self),
        }));
        if self.tail.is_none() {
            self.head = node;
            self.tail = node;
        } else {
            unsafe {
                self.tail.unwrap().as_mut().next = node;
            }
            self.tail = node;
        }
    }
    unsafe fn remove_node(&mut self, node: Option<NonNull<Node<T>>>) {
        let cur_list = node.unwrap().as_ref().list;
        assert_eq!(self as *mut _, cur_list.unwrap().as_ptr());
        if self.head == node {
            self.pop_front();
        } else if self.tail == node {
            self.pop_back();
        } else {
            node.unwrap().as_ref().next.unwrap().as_mut().prev = node.unwrap().as_ref().prev;
            node.unwrap().as_ref().prev.unwrap().as_mut().next = node.unwrap().as_ref().next;
            cur_list.unwrap().as_mut().len -= 1;
        }
        node.unwrap().as_mut().list = None;
    }

    fn push_front(&mut self, v: T) {
        let node = NonNull::new(to_raw(Node {
            value: v,
            next: self.head,
            prev: None,
            list: NonNull::new(self),
        }));
        self.push_front_node(node);
    }

    fn push_front_node(&mut self, node: Option<NonNull<Node<T>>>) {
        self.len += 1;
        unsafe {
            node.unwrap().as_mut().list = NonNull::new(self);
        }
        if self.head.is_none() {
            unsafe {
                node.unwrap().as_mut().next = None;
                node.unwrap().as_mut().prev = None;
            }
            self.head = node;
            self.tail = node;
            return;
        }
        unsafe {
            node.unwrap().as_mut().next = self.head;
            self.head.unwrap().as_mut().prev = node;
            self.head = node;
            self.head.unwrap().as_mut().prev = None;
        }
    }
    fn pop_front(&mut self) -> Option<NonNull<Node<T>>> {
        if self.head.is_none() {
            return None;
        }
        self.len -= 1;
        let ele = self.head;
        unsafe {
            self.head = ele.unwrap().as_ref().next;
            if let Some(mut h) = self.head {
                h.as_mut().prev = None;
            }
        }
        unsafe {
            ele.unwrap().as_mut().list = None;
            ele.unwrap().as_mut().prev = None;
            ele.unwrap().as_mut().next = None;
            if ele == self.tail {
                self.tail = None;
            }
        }
        ele
    }

    fn pop_back(&mut self) -> Option<NonNull<Node<T>>> {
        if self.tail.is_none() {
            return None;
        }
        self.len -= 1;
        let tail = self.tail;
        unsafe {
            self.tail = tail.unwrap().as_ref().prev;
            if let Some(mut t) = self.tail {
                t.as_mut().next = None;
            }
        }
        unsafe {
            tail.unwrap().as_mut().list = None;
            tail.unwrap().as_mut().prev = None;
            tail.unwrap().as_mut().next = None;
            if tail == self.head {
                self.head = None;
            }
        }
        tail
    }
}

/// LfuCache O(1)
pub struct LfuCache<K: Eq + Hash, V> {
    capacity: usize,
    freq_list: Option<NonNull<Link<Rc<K>>>>,
    elements: HashMap<Rc<K>, Option<NonNull<Node<Rc<K>>>>>,
    data: HashMap<Rc<K>, V>,
}

impl<K: Eq + Hash, V> Debug for LfuCache<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            let mut cur = self.freq_list;
            while let Some(c) = cur {
                write!(f, "{}: ", c.as_ref().times)?;
                writeln!(f, "{:?}", c.as_ptr())?;
                cur = c.as_ref().next;
            }
        }
        Ok(())
    }
}

impl<K: Eq + Hash, V> Drop for LfuCache<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Eq + Hash, V> LfuCache<K, V> {
    /// Returns the number of elements the cache can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of elements in the cache.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Remove all data in the cache.
    pub fn clear(&mut self) {
        let mut cur_list = self.freq_list;
        while let Some(mut l) = cur_list {
            unsafe {
                cur_list = l.as_mut().next;
                while l.as_mut().pop_back().is_some() {}
            }
        }
        for v in self.elements.values_mut() {
            unsafe {
                std::ptr::drop_in_place(v.take().unwrap().as_ptr());
            }
        }
        self.elements.clear();
        self.data.clear();
    }

    /// Create a new LfuCache with give capacity
    pub fn new(capacity: usize) -> LfuCache<K, V> {
        LfuCache {
            capacity,
            freq_list: None,
            elements: HashMap::new(),
            data: HashMap::new(),
        }
    }

    /// Return None if k doesn't exist
    pub fn get(&mut self, k: &K) -> Option<&V> {
        if !self.data.contains_key(k) {
            return None;
        }
        unsafe {
            self.update(k);
        }
        self.data.get(k)
    }

    /// Insert a new K-V entry to the cache
    pub fn insert(&mut self, k: K, v: V) {
        let k = Rc::new(k);
        if self.elements.contains_key(&k) {
            self.data.insert(k.clone(), v);
            unsafe {
                self.update(&k);
            }
            return;
        };
        unsafe {
            self.eviction();
        }
        let n = NonNull::new(to_raw(Node {
            prev: None,
            next: None,
            list: None,
            value: k.clone(),
        }));
        self.elements.insert(k.clone(), n);
        self.data.insert(k.clone(), v);
        if self.freq_list.is_none() {
            self.freq_list = NonNull::new(to_raw(Link::new(1)));
        }
        unsafe {
            if self.freq_list.unwrap().as_ref().times != 1 {
                let once = NonNull::new(to_raw(Link::new(1)));
                once.unwrap().as_mut().next = self.freq_list;
                self.freq_list.unwrap().as_mut().prev = once;
                self.freq_list = once;
            }
            self.freq_list.unwrap().as_mut().push_front_node(n);
        }
    }
}

impl<K: Eq + Hash, V> LfuCache<K, V> {
    unsafe fn update(&mut self, k: &K) {
        let &node = self.elements.get(k).unwrap();
        let mut cur_list = node.unwrap().as_ref().list.unwrap();
        // Remove node from original list
        cur_list.as_mut().remove_node(node);

        let cur_cnt = cur_list.as_ref().times;

        if cur_list.as_ref().next.is_none() {
            // Create new list, append list
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = Some(cur_list);
            cur_list.as_mut().next = NonNull::new(list);
        }

        if cur_list.as_ref().next.unwrap().as_ref().times != cur_cnt + 1 {
            // Create and insert new list
            let next = cur_list.as_ref().next;
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = Some(cur_list);
            (*list).next = next;
            let list = NonNull::new(list);
            cur_list.as_mut().next = list;
            next.unwrap().as_mut().prev = list;
        }

        let old_list = cur_list;
        let next_list = cur_list.as_ref().next;

        // Add node to next list
        next_list.unwrap().as_mut().push_front_node(node);

        if old_list.as_ref().len == 0 {
            if old_list == self.freq_list.unwrap() {
                // Remove empty head list
                self.freq_list = old_list.as_ref().next;
            } else if old_list.as_ref().next.is_none() {
                // Unreachable
                unreachable!();
            } else {
                old_list.as_ref().prev.unwrap().as_mut().next = old_list.as_ref().next;
                old_list.as_ref().next.unwrap().as_mut().prev = old_list.as_ref().prev;
            }
            std::ptr::drop_in_place(old_list.as_ptr());
        }
    }
    unsafe fn eviction(&mut self) {
        if self.data.len() == self.capacity {
            let ptr = self.freq_list.unwrap().as_mut().pop_back();
            let k = ptr.unwrap().as_ref().value.clone();
            self.elements.remove(&k);
            self.data.remove(&k);
            std::ptr::drop_in_place(ptr.unwrap().as_ptr());

            // Remove empty list
            if self.freq_list.unwrap().as_ref().len == 0 {
                let empty_head = self.freq_list;
                self.freq_list = self.freq_list.unwrap().as_ref().next;
                std::ptr::drop_in_place(empty_head.unwrap().as_ptr());
            }
        }
    }
}
