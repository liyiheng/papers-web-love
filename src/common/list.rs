#![allow(dead_code)]
#![deny(missing_docs)]
use std::fmt::Debug;
use std::ptr::NonNull;

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
