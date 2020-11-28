#![allow(dead_code)]
#[test]
fn test_list() {
    unsafe {
        let mut list = Link::<i32>::new(123);
        list.push_front(1);
        list.push_front(2);
        list.push_back(3);
        list.push_back(4);
        // [2,1,3,4]
        assert_eq!(2, (*list.head).value);
        assert_eq!(1, (*(*list.head).next).value);
        assert_eq!(3, (*(*list.tail).prev).value);
        assert_eq!(4, (*list.tail).value);
        assert_eq!(3, (*(*(*list.head).next).next).value);
    }
}
#[test]
fn test_lfu_robust() {
    let capacity = 10;
    let mut cache = LfuCache::new(capacity);
    for i in 0..capacity * 100 {
        cache.insert(i, i);
    }
    for i in 0..capacity * 100 {
        cache.insert(i % 3, i);
        cache.insert(i % 5, i);
    }
    for _ in 0..100 {
        for i in 0..capacity {
            cache.get(&i);
        }
    }
}

#[test]
fn test_lfu() {
    let capacity = 10;
    let mut cache = LfuCache::new(capacity);
    for i in 0..capacity {
        cache.insert(i, i);
        assert_eq!(cache.get(&i), Some(&i));
    }
    for i in 0..capacity {
        assert_eq!(cache.get(&i), Some(&i));
    }
    assert_eq!(cache.get(&5), Some(&5));
    assert_eq!(cache.get(&5), Some(&5));
    assert_eq!(cache.get(&5), Some(&5));
    assert_eq!(cache.get(&6), Some(&6));
    assert_eq!(cache.get(&6), Some(&6));
    for i in 2..capacity {
        cache.insert(i, i);
    }
    assert_eq!(cache.get(&5), Some(&5));
    assert_eq!(cache.get(&6), Some(&6));
}

#[inline]
fn to_raw<T>(t: T) -> *mut T {
    Box::leak(Box::new(t))
}

use std::collections::HashMap;

// TODO Option<NonNull<Node<T>>>
#[derive(Debug)]
struct Node<T> {
    prev: *mut Node<T>,
    next: *mut Node<T>,
    list: *mut Link<T>,
    value: T,
}
#[derive(Debug)]
struct Link<T> {
    times: u64,
    len: usize,
    head: *mut Node<T>,
    tail: *mut Node<T>,
    prev: *mut Link<T>,
    next: *mut Link<T>,
}
impl<T> Link<T> {
    fn new(times: u64) -> Link<T> {
        Link {
            times,
            len: 0,
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }
    }
    fn push_back(&mut self, v: T) {
        self.len += 1;
        let node = Box::leak(Box::new(Node {
            value: v,
            next: std::ptr::null_mut(),
            prev: self.tail,
            list: self,
        }));
        if self.tail.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            unsafe {
                (*self.tail).next = node;
            }
            self.tail = node;
        }
    }

    fn push_front(&mut self, v: T) {
        let node = Box::leak(Box::new(Node {
            value: v,
            next: self.head,
            prev: std::ptr::null_mut(),
            list: self,
        }));
        self.push_front_node(node);
    }

    fn push_front_node(&mut self, node: *mut Node<T>) {
        self.len += 1;

        unsafe {
            (*node).list = self;
        }
        if self.head.is_null() {
            self.head = node;
            self.tail = node;
            return;
        }
        unsafe {
            (*self.head).prev = node;
            self.head = (*self.head).prev;
        }
    }
    fn pop_front(&mut self) -> *mut Node<T> {
        if self.head.is_null() {
            return std::ptr::null_mut();
        }
        self.len -= 1;
        let head = self.head;
        unsafe {
            self.head = (*head).next;
            (*self.head).prev = std::ptr::null_mut();
        }
        head
    }

    fn pop_back(&mut self) -> *mut Node<T> {
        if self.tail.is_null() {
            return std::ptr::null_mut();
        }
        self.len -= 1;
        let tail = self.tail;
        unsafe {
            self.tail = (*tail).prev;
            (*self.tail).next = std::ptr::null_mut();
        }
        tail
    }
}
// TODO Iter
// TODO remove(&k);
pub struct LfuCache<K: Eq + std::hash::Hash, V> {
    capacity: usize,
    freq_list: *mut Link<K>,            // TODO Rc<K>
    elements: HashMap<K, *mut Node<K>>, // TODO Rc<K>
    data: HashMap<K, V>,                // TODO Rc<K>
}
// TODO remove Clone
impl<K: Clone + Eq + std::hash::Hash, V> LfuCache<K, V> {
    pub fn new(capacity: usize) -> LfuCache<K, V> {
        LfuCache {
            capacity,
            freq_list: std::ptr::null_mut(),
            elements: HashMap::new(),
            data: HashMap::new(),
        }
    }
    pub fn get(&mut self, k: &K) -> Option<&V> {
        if !self.data.contains_key(k) {
            return None;
        }
        unsafe {
            self.update(k);
        }
        self.data.get(k)
    }
    unsafe fn update(&mut self, k: &K) {
        let node = {
            // Remove node from original list
            let &node = self.elements.get(k).unwrap();
            if (*node).next.is_null() {
                if !(*node).prev.is_null() {
                    let n = (*(*node).list).pop_back();
                    n
                } else {
                    node
                }
            } else if (*node).prev.is_null() {
                (*(*node).list).pop_front()
            } else {
                (*(*node).next).prev = (*node).prev;
                (*(*node).prev).next = (*node).next;
                (*(*node).list).len -= 1;
                node
            }
        };
        let cur_cnt = (*(*node).list).times;
        if (*(*node).list).next.is_null() {
            // Create new list, append list
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = (*node).list;
            (*(*node).list).next = list;
        }
        if (*(*(*node).list).next).times != cur_cnt + 1 {
            // Create and insert new list
            let next = (*(*node).list).next;
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = (*node).list;
            (*list).next = next;
            (*(*node).list).next = list;
            (*next).prev = list;
        }
        let old_list = (*node).list;
        // Add node to next list
        (*(*(*node).list).next).push_front_node(node);
        if (*old_list).len == 0 {
            if old_list == self.freq_list {
                // Remove empty head list
                self.freq_list = (*old_list).next;
            } else if (*old_list).next.is_null() {
                // Unreachable
            } else {
                (*(*old_list).prev).next = (*old_list).next;
                (*(*old_list).next).prev = (*old_list).prev;
                std::ptr::drop_in_place(old_list);
            }
        }
    }
    pub fn insert(&mut self, k: K, v: V) {
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
        let n = Box::leak(Box::new(Node {
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            list: std::ptr::null_mut(),
            value: k.clone(),
        }));
        self.elements.insert(k.clone(), n);
        self.data.insert(k.clone(), v);
        if self.freq_list.is_null() {
            self.freq_list = to_raw(Link::new(1));
        }
        unsafe {
            (*self.freq_list).push_front_node(n);
        }
    }
    unsafe fn eviction(&mut self) {
        if self.data.len() == self.capacity {
            let ptr = (*self.freq_list).pop_back();
            let k = &(*ptr).value;
            self.data.remove(k);
            self.elements.remove(k);
            std::ptr::drop_in_place(ptr);
            if (*self.freq_list).len == 0 {
                let empty_head = self.freq_list;
                self.freq_list = (*self.freq_list).next;
                std::ptr::drop_in_place(empty_head);
            }
        }
    }
}
