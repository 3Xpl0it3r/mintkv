use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::ops::DerefMut;
use std::rc::Rc;
use std::usize;

use crate::constant::{DEFAULT_META_PN, DEFAULT_PAGE_SIZE};
use crate::error::Error;
use crate::freelist::Freelist;
use crate::meta::Meta;
use crate::node::{self, Item, Node};
use crate::pager::Pager;

pub struct BTree {
    pub pager: Rc<Pager>,
    pub metadata: Meta,
    pub freelist: Freelist,
}

impl BTree {
    pub fn new(path: &str) -> Self {
        let mut should_initial = false;
        let fp = match OpenOptions::new().write(true).read(true).open(path) {
            Ok(file_ptr) => file_ptr,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    should_initial = true;
                    File::create_new(path).expect("crate new database failed")
                } else {
                    panic!("open database failed");
                }
            }
        };
        let pager = Pager::new(fp);
        let mut metadata = Meta::default();
        let mut freelist = Freelist::default();
        if should_initial {
            let mut meta_page = pager.allocate_page(DEFAULT_META_PN);
            metadata.serialize(&mut meta_page.data);
            pager.write_page(&meta_page);

            let mut fls_page = pager.allocate_page(freelist.get_next_page());
            freelist.serialize(&mut fls_page.data);
            pager.write_page(&fls_page);
        } else {
            let mta_page = pager.read_page(DEFAULT_META_PN).unwrap();
            metadata.deserialize(&mta_page.data);

            let fls_page = pager.read_page(metadata.freelist_page).unwrap();
            freelist.deserialize(&fls_page.data);
        }
        BTree {
            pager: Rc::new(pager),
            metadata,
            freelist,
        }
    }

    pub fn display(&self) {
        if self.metadata.root == 0 {
            return;
        }
        let mut queue = vec![self.metadata.root];
        let mut hight = 0;
        while !queue.is_empty() {
            hight += 1;
            if hight > 4 {
                return;
            }
            println!("High-{}", hight);
            let count = queue.len();
            for _ in 0..count {
                let node_ptr = queue.remove(0);
                let node = self.get_node(node_ptr).unwrap();
                node.display();
                if node.children.is_empty() {
                    continue;
                }
                queue.extend(node.children);
            }
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        let item = Item::new(key.as_bytes().into(), value.as_bytes().into());
        if self.metadata.root == 0 {
            let mut node_page = self.pager.allocate_page(self.freelist.get_next_page());
            let mut new_node = Node::new_empty_node(node_page.page_number);
            new_node.items.push(item);
            new_node.serialize(&mut node_page.data);

            self.write_node(&mut new_node);
            self.metadata.root = new_node.page_number;
            return;
        }

        let mut parent_indices = vec![0];
        let (mut node, index, found) = self
            .find_node(self.metadata.root, key, &mut parent_indices)
            .unwrap();

        if found {
            node.items[index] = item;
        } else {
            node.items.insert(index, item);
        }

        let mut ancestors = self.get_nodes(&parent_indices);
        if ancestors.len() > 1 {
            // for last modified node has not persistent into disk;
            let last = ancestors.len() - 1;
            ancestors.remove(last);
        }
        if node.page_number == self.metadata.root {
            ancestors[0] = Rc::new(RefCell::new(node));
        } else if node.is_overflow() {
            ancestors.push(Rc::new(RefCell::new(node)));
        } else {
            self.write_node(&mut node);
        }

        for i in (0..ancestors.len() - 1).rev() {
            let parent = ancestors[i].clone();
            let child = ancestors[i + 1].clone();
            let child_index = parent_indices[i + 1];
            if child.borrow().is_overflow() {
                let (middile_item, mut sibling) = child.borrow_mut().split().unwrap();
                sibling.page_number = self.freelist.get_next_page();
                parent.borrow_mut().items.insert(child_index, middile_item);
                parent
                    .borrow_mut()
                    .children
                    .insert(child_index + 1, sibling.page_number);
                self.write_node(&mut child.borrow_mut());
                self.write_node(&mut sibling);
            } else {
                self.write_node(&mut child.borrow_mut());
            }
        }

        let root_node = ancestors[0].clone();

        if root_node.borrow().is_overflow() {
            let mut new_root = Node::default();
            let (middle_item, mut sibling) = root_node.borrow_mut().split().unwrap();
            sibling.page_number = self.freelist.get_next_page();
            new_root.items.push(middle_item);
            new_root.children.push(root_node.borrow().page_number);
            new_root.children.push(sibling.page_number);
            self.write_nodes(
                vec![&mut new_root, &mut root_node.borrow_mut(), &mut sibling].deref_mut(),
            );
            self.metadata.root = new_root.page_number;
        } else {
            self.write_node(&mut root_node.borrow_mut())
        }
    }

    pub fn delete(&mut self, key: &str) -> Result<String, Error> {
        if self.metadata.root == 0 {
            return Err(Error::EmptyTree);
        }
        let mut ancestor_idx = vec![0];
        let (mut removed_node, removed_index, found) = self
            .find_node(self.metadata.root, key, &mut ancestor_idx)
            .unwrap();
        if !found {
            return Err(Error::KeyNotFound);
        }
        let removed_value = removed_node.items[removed_index].clone();

        if removed_node.is_leaf() == 1 {
            removed_node.items.remove(removed_index);
            self.write_node(&mut removed_node);
        } else {
            let mut affected = self.remove_from_internal_node(&mut removed_node, removed_index);
            ancestor_idx.append(&mut affected);
        }
        let ancestors = self.get_nodes(&ancestor_idx);
        for i in (0..ancestors.len() - 1).rev() {
            let parent = ancestors[i].clone();
            let child = ancestors[i + 1].clone();
            let child_index = ancestor_idx[i + 1];
            if child.borrow().is_underflow() {
                self.reblance(
                    &mut parent.borrow_mut(),
                    &mut child.borrow_mut(),
                    child_index,
                )
            } else {
                self.write_node(&mut child.borrow_mut());
            }
        }

        let root_node = ancestors.first().unwrap();
        if root_node.borrow_mut().items.is_empty() && !root_node.borrow_mut().children.is_empty() {
            self.metadata.root = root_node.borrow_mut().children[0];
            self.delete_node(root_node.borrow().page_number);
        } else {
            self.write_node(&mut root_node.borrow_mut());
        }

        Ok(String::from_utf8(removed_value.value).unwrap())
    }

    fn reblance(
        &mut self,
        parent_node: &mut Node,
        deficient_node: &mut Node,
        deficient_index: usize,
    ) {
        if deficient_index > 0 {
            // if deficient node's left sibling exists and has more than minimum number of
            // elements, then rotate right
            let mut l_sibling = self
                .get_node(parent_node.children[deficient_index - 1])
                .unwrap();
            if l_sibling.can_spare_element() {
                let separator_item = parent_node.items.remove(deficient_index - 1);
                deficient_node.items.insert(0, separator_item);

                let left_item = l_sibling.items.pop().unwrap();
                parent_node.items.insert(deficient_index - 1, left_item);
                if l_sibling.is_leaf() == 0 {
                    let child = l_sibling.children.pop().unwrap();
                    deficient_node.children.insert(0, child);
                }
                self.write_node(&mut l_sibling);
                self.write_node(deficient_node);
                return;
            }
        }

        if deficient_index < parent_node.children.len() - 1 {
            // borrow from right
            // if deficient node's right sibling exists and has more than minimum number of
            // elements, then rotate left
            let mut r_sibling = self
                .get_node(parent_node.children[deficient_index + 1])
                .unwrap();
            if r_sibling.can_spare_element() {
                let separator_item = parent_node.items.remove(deficient_index);
                deficient_node.items.push(separator_item);

                let right_item = r_sibling.items.remove(0);
                parent_node.items.insert(deficient_index, right_item);
                if r_sibling.is_leaf() == 0 {
                    let child = r_sibling.children.remove(0);
                    deficient_node.children.push(child);
                }

                self.write_node(&mut r_sibling);
                self.write_node(deficient_node);
                return;
            }
        }
        // immediate sibling have only the minimum number of elements, then merge with a sibling
        // sandwiching their seperator take off from their parents
        if deficient_index == 0 {
            if let Some(mut r_sibling) = self.get_node(parent_node.children[1]) {
                let separator = parent_node.items.remove(0);
                deficient_node.items.push(separator);
                deficient_node.items.append(&mut r_sibling.items);
                if r_sibling.is_leaf() == 0 {
                    deficient_node.children.append(&mut r_sibling.children);
                }
                parent_node.children.remove(1);
                self.write_node(deficient_node);
                self.delete_node(r_sibling.page_number);
            }
        } else {
            let mut l_sibling = self
                .get_node(parent_node.children[deficient_index - 1])
                .unwrap();
            let separator_item = parent_node.items.remove(deficient_index - 1);
            l_sibling.items.push(separator_item);
            l_sibling.items.append(&mut deficient_node.items);
            if deficient_node.is_leaf() == 0 {
                l_sibling.children.append(&mut deficient_node.children);
            }
            parent_node.children.remove(deficient_index);
            self.write_node(&mut l_sibling);
            self.delete_node(deficient_node.page_number);
        }
    }

    fn remove_from_internal_node(&mut self, node: &mut Node, removed_index: usize) -> Vec<usize> {
        let mut affected_nodes = vec![removed_index];

        // get the largest node in left child
        let mut child_offset = node.children[removed_index];
        let mut child_node: Node;
        loop {
            child_node = self.get_node(child_offset).unwrap();
            if child_node.is_leaf() == 1 {
                break;
            }
            let travel_index = child_node.children.len() - 1;
            child_offset = child_node.children[travel_index];
            affected_nodes.push(travel_index);
        }
        let new_seperator_item = child_node.items.pop().unwrap();
        node.items[removed_index] = new_seperator_item;
        self.write_node(node);
        self.write_node(&mut child_node);
        affected_nodes
    }

    fn get_nodes(&self, indexes: &[usize]) -> Vec<Rc<RefCell<Node>>> {
        let mut nodes = vec![];
        let root = self.get_node(self.metadata.root).unwrap();
        nodes.push(Rc::new(RefCell::new(root)));
        if indexes.len() == 1 {
            return nodes;
        }

        for i in 1..indexes.len() {
            let child_offset = nodes[i - 1].clone().borrow().children[indexes[i]];
            let child_node = self.get_node(child_offset).unwrap();
            nodes.push(Rc::new(RefCell::new(child_node)));
        }

        nodes
    }

    pub fn find(&self, key: &str) -> Result<Item, Error> {
        if self.metadata.root == 0 {
            return Err(Error::EmptyTree);
        }

        let mut ancestors = vec![];
        if let Ok((node, index, found)) = self.find_node(self.metadata.root, key, &mut ancestors) {
            if found {
                return Ok(node.items[index].clone());
            }
        }
        Err(Error::KeyNotFound)
    }

    fn find_node(
        &self,
        node_offset: u64,
        key: &str,
        ancestors: &mut Vec<usize>,
    ) -> Result<(Node, usize, bool), Error> {
        let node: Node = if let Some(node) = self.get_node(node_offset) {
            node
        } else {
            return Err(Error::PageLoadErr);
        };

        let (found, index) = node.find_key(key);
        if node.is_leaf() == 1 || found {
            // node.item[index-1] < key < node.item[index]
            return Ok((node, index, found));
        }

        // node.
        ancestors.push(index);
        self.find_node(node.children[index], key.clone(), ancestors)
    }

    fn get_node(&self, page_number: u64) -> Option<Node> {
        let mut node = Node::default();
        let node_page = self.pager.read_page(page_number).unwrap();
        node.deserialize(&node_page.data);
        node.page_number = page_number;
        Some(node)
    }

    pub fn write_nodes(&mut self, nodes: &mut [&mut Node]) {
        for node in nodes {
            self.write_node(node)
        }
    }

    pub fn write_node(&mut self, node: &mut Node) {
        if node.page_number == 0 {
            node.page_number = self.freelist.get_next_page();
        }
        let mut page = self.pager.allocate_page(node.page_number);

        node.serialize(&mut page.data);
        self.pager.write_page(&page);
    }

    pub fn delete_node(&mut self, node: u64) {
        if let Some(mut page) = self.pager.read_page(node) {
            page.data[0..DEFAULT_PAGE_SIZE].clone_from_slice(vec![0; DEFAULT_PAGE_SIZE].as_ref());
            self.pager.write_page(&page);
        }
        self.freelist.release_page(node);
    }
}

impl Drop for BTree {
    fn drop(&mut self) {
        let mut meta_page = self.pager.allocate_page(DEFAULT_META_PN);
        self.metadata.serialize(&mut meta_page.data);
        self.pager.write_page(&meta_page);

        let mut fls_page = self.pager.allocate_page(self.metadata.freelist_page);
        self.freelist.serialize(&mut fls_page.data);
        self.pager.write_page(&fls_page);
    }
}
