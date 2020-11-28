#![allow(dead_code)]
use std::collections::HashMap;
#[test]
fn test_list() {
    unsafe {
        let mut list = Link::<i32>::new(123);
        list.push_front(1);
        list.push_front(2);
        list.push_back(3);
        list.push_back(4);
        // [2,1,3,4]
        assert_eq!(list.len, 4);
        assert_eq!(2, (*list.head).value);
        assert_eq!(1, (*(*list.head).next).value);
        assert_eq!(3, (*(*list.tail).prev).value);
        assert_eq!(4, (*list.tail).value);
        assert_eq!(3, (*(*(*list.head).next).next).value);

        list.remove_node((*(*list.head).next).next);
        // [2,1,4]
        assert_eq!(list.len, 3);
        assert_eq!(4, (*(*(*list.head).next).next).value);
        assert_eq!(2, (*list.head).value);
        assert_eq!(1, (*(*list.head).next).value);
        assert_eq!(4, (*list.tail).value);
        list.remove_node(list.head);
        // [1,4]
        assert_eq!(list.len, 2);
        assert_eq!(1, (*list.head).value);
        assert_eq!(1, (*(*list.tail).prev).value);
        assert_eq!(4, (*list.tail).value);
        assert_eq!(4, (*(*list.head).next).value);
        list.remove_node(list.tail);
        // [1]
        assert_eq!(list.len, 1);
        assert_eq!(1, (*list.head).value);
        assert_eq!(1, (*list.tail).value);
        assert_eq!(list.head, list.tail);
        assert!((*list.head).prev.is_null());
        assert!((*list.head).next.is_null());
        list.remove_node(list.tail);
        // []
        assert_eq!(list.len, 0);
        assert!(list.head.is_null());
        assert!(list.tail.is_null());
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

// #[test]
fn test_mem_leak() {
    let mut cache = LfuCache::new(100);
    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_secs(60);
    while start.elapsed() < dur {
        for i in 0..10000 {
            let k = format!("This is a long key ..................{}", i);
            cache.insert(k.clone(), k);
        }
        for i in (0..10000).rev() {
            let k = format!("This is a long key ..................{}", i);
            cache.insert(k.clone(), k);
        }
        for i in 0..10000 {
            let k = format!("This is a long key ..................{}", i);
            cache.insert(k.clone(), k);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[inline]
fn to_raw<T>(t: T) -> *mut T {
    Box::leak(Box::new(t))
}
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
    fn print(&self) {
        // TODO impl Debug
        unsafe {
            println!("List {:?}", self as *const _);
            let mut c = self.head;
            while !c.is_null() {
                print!("{:?} -> ", c);
                c = (*c).next;
            }
            println!("NULL");
            let mut c = self.tail;
            while !c.is_null() {
                print!("{:?} -> ", c);
                c = (*c).prev;
            }
            println!("NULL");
        }
    }

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
    unsafe fn remove_node(&mut self, node: *mut Node<T>) {
        let cur_list = (*node).list;
        assert_eq!(self as *mut _, cur_list);
        if self.head == node {
            self.pop_front();
        } else if self.tail == node {
            self.pop_back();
        } else {
            (*(*node).next).prev = (*node).prev;
            (*(*node).prev).next = (*node).next;
            (*cur_list).len -= 1;
        }
        (*node).list = std::ptr::null_mut();
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
            unsafe {
                (*node).next = std::ptr::null_mut();
                (*node).prev = std::ptr::null_mut();
            }
            self.head = node;
            self.tail = node;
            return;
        }
        unsafe {
            (*node).next = self.head;
            (*self.head).prev = node;
            self.head = node;
            (*self.head).prev = std::ptr::null_mut();
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
            if !self.head.is_null() {
                (*self.head).prev = std::ptr::null_mut();
            }
        }
        unsafe {
            (*head).list = std::ptr::null_mut();
            (*head).prev = std::ptr::null_mut();
            (*head).next = std::ptr::null_mut();
            if head == self.tail {
                self.tail = std::ptr::null_mut();
            }
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
            if !self.tail.is_null() {
                (*self.tail).next = std::ptr::null_mut();
            }
        }
        unsafe {
            (*tail).list = std::ptr::null_mut();
            (*tail).prev = std::ptr::null_mut();
            (*tail).next = std::ptr::null_mut();
            if tail == self.head {
                self.head = std::ptr::null_mut();
            }
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
        let &node = self.elements.get(k).unwrap();
        let cur_list = (*node).list;
        // Remove node from original list
        (*cur_list).remove_node(node);
        let cur_cnt = (*cur_list).times;
        if (*cur_list).next.is_null() {
            // Create new list, append list
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = cur_list;
            (*cur_list).next = list;
        }
        if (*(*cur_list).next).times != cur_cnt + 1 {
            // Create and insert new list
            let next = (*cur_list).next;
            let list = to_raw(Link::new(cur_cnt + 1));
            (*list).prev = cur_list;
            (*list).next = next;
            (*cur_list).next = list;
            (*next).prev = list;
        }
        let old_list = cur_list;
        let next_list = (*cur_list).next;
        // Add node to next list
        (*next_list).push_front_node(node);
        if (*old_list).len == 0 {
            if old_list == self.freq_list {
                // Remove empty head list
                self.freq_list = (*old_list).next;
            } else if (*old_list).next.is_null() {
                // Unreachable
                unreachable!();
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
            if (*self.freq_list).times != 1 {
                let once = to_raw(Link::new(1));
                (*once).next = self.freq_list;
                (*self.freq_list).prev = once;
                self.freq_list = once;
            }
            (*self.freq_list).push_front_node(n);
        }
    }
    unsafe fn eviction(&mut self) {
        if self.data.len() == self.capacity {
            let ptr = (*self.freq_list).pop_back();
            let k = &(*ptr).value;
            self.elements.remove(k);
            self.data.remove(k);
            std::ptr::drop_in_place(ptr);
            if (*self.freq_list).len == 0 {
                let empty_head = self.freq_list;
                self.freq_list = (*self.freq_list).next;
                std::ptr::drop_in_place(empty_head);
            }
        }
    }
    fn print(&self) {
        // TODO impl Debug
        if 2 > 1 {
            return;
        }
        unsafe {
            let mut c = self.freq_list;
            while !c.is_null() {
                print!("{}: ", (*c).times);
                (*c).print();
                c = (*c).next;
            }
        }
    }
}
