use std::cell::RefCell;
use std::rc::Rc;
use std::usize;

use crate::bytes;
use crate::errors::Error;
use crate::util::Random;

const MAX_SKIP_HEIGH: usize = 16;

// SkipList[#TODO] (shoule add some comments )
#[derive(Debug)]
pub(super) struct SkipList {
    head: SkipNode,
    height: i32,
}

// Default[#TODO] (should add some comments)
impl Default for SkipList {
    fn default() -> Self {
        SkipList {
            head: SkipNode::default(),
            height: 1,
        }
    }
}

pub type SkipNode = Rc<RefCell<Node>>;
const EMPTY_NODE: Option<Rc<RefCell<Node>>> = None;

#[derive(Debug, Default, PartialEq)]
pub struct Node {
    key: Vec<u8>,
    value: Vec<u8>,
    next_nodes: [Option<Rc<RefCell<Node>>>; MAX_SKIP_HEIGH],
}

fn get_random_height() -> i32 {
    let mut height = 0;
    for _ in 0..MAX_SKIP_HEIGH {
        let rand_dome = Random::u32().unwrap();
        let f_n = rand_dome as f64 / u32::MAX as f64;
        if f_n < 0.5 {
            height += 1;
        }
    }
    height
}

// SkipList[#TODO] (should add some comments)
impl SkipList {
    pub(super) fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let (mut travels, maybe_found) = self.search(key);
        if maybe_found.is_some() {
            return Err(Error::KeyExists);
        }

        let height = get_random_height();
        let new_node = Rc::new(RefCell::new(Node {
            key: key.into(),
            value: value.into(),
            next_nodes: [EMPTY_NODE; MAX_SKIP_HEIGH],
        }));

        for level in 0..height {
            let prev_node: SkipNode;
            if let Some(node) = travels[level as usize].take() {
                prev_node = node;
            } else {
                prev_node = self.head.clone();
            }
            new_node.borrow_mut().next_nodes[level as usize] =
                prev_node.borrow_mut().next_nodes[level as usize].take();
            prev_node.borrow_mut().next_nodes[level as usize] = Some(new_node.clone());
        }

        if height > self.height {
            self.height = height;
        }
        Ok(())
    }

    pub(super) fn delete(&mut self, key: &[u8]) -> Result<Vec<u8>, Error> {
        let (mut travels, may_found) = self.search(key);
        if may_found.is_none() {
            return Err(Error::KeyNotFound);
        }

        for level in 0..self.height {
            let prev_node = travels[level as usize].take().unwrap();
            let may_del = prev_node.borrow().next_nodes[level as usize].clone();
            if may_del.is_none() {
                continue;
            }
            let del_node = may_del.unwrap();
            if del_node.borrow().key == key {
                prev_node.borrow_mut().next_nodes[level as usize] =
                    del_node.borrow_mut().next_nodes[level as usize].take();
            }
        }

        // resize the height
        for level in (0..self.height).rev() {
            if self.head.borrow().next_nodes[level as usize].is_none() {
                self.height -= 1;
            }
        }

        let node = Rc::try_unwrap(may_found.unwrap()).unwrap().into_inner();
        Ok(node.value)
    }

    pub(super) fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, may_found) = self.search(key);
        match may_found {
            Some(found) => Ok(found.borrow().value.clone()),
            None => Err(Error::KeyNotFound),
        }
    }

    fn search(&self, key: &[u8]) -> ([Option<SkipNode>; MAX_SKIP_HEIGH], Option<SkipNode>) {
        let mut travels = [EMPTY_NODE; MAX_SKIP_HEIGH];
        let mut prev_node: SkipNode = self.head.clone();
        let mut next: Option<SkipNode> = None;
        for level in (0..self.height).rev() {
            loop {
                let may_next = prev_node.borrow().next_nodes[level as usize].clone();
                if let Some(node) = may_next.as_ref() {
                    match bytes::compare(key, &node.borrow().key) {
                        std::cmp::Ordering::Less => break,
                        std::cmp::Ordering::Equal => {
                            next = Some(node.clone());
                            break;
                        }
                        std::cmp::Ordering::Greater => prev_node = node.clone(),
                    }
                } else {
                    break;
                }
            }
            travels[level as usize] = Some(prev_node.clone());
        }
        if let Some(node) = next.as_ref() {
            if bytes::compare(&node.borrow().key, key) == std::cmp::Ordering::Equal {
                return (travels, Some(node.clone()));
            }
        }
        (travels, None)
    }
}

pub(super) struct IntoIterator(SkipList);

impl SkipList {
    pub(super) fn into_iter(self) -> IntoIterator {
        IntoIterator(self)
    }
}

impl Iterator for IntoIterator {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let drop = self.0.head.borrow_mut().next_nodes[0].take();
        drop.as_ref()?;
        let node = drop.unwrap();
        let next = node.borrow_mut().next_nodes[0].take();
        self.0.head.borrow_mut().next_nodes[0] = next;
        /* let node = Rc::try_unwrap(node).unwrap().into_inner(); */
        let key = node.borrow().key.clone();
        let value = node.borrow().value.clone();
        Some((key,value))
        /* if let Some(node) = self.0.head.borrow_mut().next_nodes[0].take() {
            let next = node.borrow_mut().next_nodes[0].take();
            self.0.head.borrow_mut().next_nodes[0] = next;
            let node = Rc::try_unwrap(node).unwrap().into_inner();
            Some((node.key, node.value))
        } else {
            None
        } */
    }
}

// Iter<'a>[#TODO] (shoule add some comments )
pub(super) struct Iter {
    next: Option<SkipNode>,
}

impl SkipList {
    pub(super) fn iter(&self) -> Iter {
        Iter {
            next: self.head.borrow().next_nodes[0].clone(),
        }
    }
}

// Iterator[#TODO] (should add some comments)
impl Iterator for Iter {
    type Item = SkipNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.borrow().next_nodes[0].clone();
            node
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut list = SkipList::default();
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];

        assert_eq!(list.get(&key), Err(Error::KeyNotFound));

        list.insert(&key, &value).unwrap();

        assert_eq!(list.get(&key), Ok(value));
    }

    #[test]
    fn test_insert_and_delete() {
        let mut list = SkipList::default();
        let key = vec![1, 2, 3];
        let value = vec![4, 5, 6];

        assert_eq!(list.delete(&key), Err(Error::KeyNotFound));

        list.insert(&key, &value).unwrap();

        assert_eq!(list.delete(&key), Ok(value));
        assert_eq!(list.get(&key), Err(Error::KeyNotFound));
    }

    #[test]
    fn test_iterator() {
        let mut list = SkipList::default();
        let pairs = vec![(vec![1], vec![2]), (vec![3], vec![4]), (vec![5], vec![6])];

        for (key, value) in &pairs {
            list.insert(key, value).unwrap();
        }

        let mut iter = list.iter();

        for (key, value) in pairs {
            let node = iter.next().unwrap();
            assert_eq!(node.borrow().key, key);
            assert_eq!(node.borrow().value, value);
        }

        assert_eq!(iter.next(), None);
    }
}
