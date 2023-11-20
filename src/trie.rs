use crate::constant::LEAF_FLAG;
use crate::nibble::Nibble;
use crate::node::{empty_children, BranchNode, Node};

#[derive(Default)]
pub struct PatriciaTrie {
    root: Node,
}

pub trait Trie {
    /// Returns the value for key stored in the trie.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Checks that the key is present in the trie
    fn contains(&self, key: &[u8]) -> bool;

    /// Inserts value into trie and modifies it if it exists
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>);

    /// Removes any existing value for key from the trie.
    fn remove(&mut self, key: &[u8]) -> bool;
}

impl Trie for PatriciaTrie {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        Self::get_at(&self.root, &Nibble::from_raw(key.to_vec(), false))
    }

    fn contains(&self, key: &[u8]) -> bool {
        Self::get_at(&self.root, &Nibble::from_raw(key.to_vec(), false)).is_some()
    }

    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.root = Self::insert_at(self.root.clone(), Nibble::from_raw(key, true), value);
    }

    fn remove(&mut self, key: &[u8]) -> bool {
        let (new_root, deleted) =
            Self::delete_at(self.root.clone(), &Nibble::from_raw(key.to_vec(), true));
        self.root = new_root;
        deleted
    }
}

impl PatriciaTrie {
    pub fn new() -> Self {
        PatriciaTrie { root: Node::Empty }
    }

    fn insert_at(node: Node, key: Nibble, value: Vec<u8>) -> Node {
        match node {
            Node::Empty => Node::leaf(key, value),
            Node::Leaf(old_leaf) => {
                let mut old_node = old_leaf.borrow_mut();
                let old_key = old_node.nibble.clone();
                let old_value = old_node.value.clone();
                let next_match_index = key.match_len(&old_key);
                let leaf_index = next_match_index + 1;
                //match all, replace value
                if next_match_index == old_key.len() {
                    old_node.value = value;
                    return Node::Leaf(old_leaf.clone());
                }
                let mut branch = BranchNode {
                    child: empty_children(),
                    value: None,
                };
                branch.insert_at(
                    old_key.value_at(next_match_index),
                    Node::leaf(old_key.slice_from(leaf_index), old_value),
                );

                branch.insert_at(
                    key.value_at(next_match_index),
                    Node::leaf(key.slice_from(leaf_index), value),
                );
                //has no match, branch node
                if next_match_index == 0 {
                    return Node::branch_with_param(branch);
                }
                Node::extension(
                    key.sub_slice(0, next_match_index),
                    Node::branch_with_param(branch),
                )
            }
            Node::Extension(old_extension) => {
                let mut old_node = old_extension.borrow_mut();
                let old_key = &old_node.nibble;
                let next = old_node.next.clone();
                let next_match_index = key.match_len(old_key);
                //extension not match, add branch
                if next_match_index == 0 {
                    let old_next = if old_key.len() == 1 {
                        next
                    } else {
                        Node::extension(old_key.slice_from(1), next)
                    };
                    let mut branch = BranchNode::new();
                    branch.insert_at(old_key.value_at(0), old_next);
                    return Self::insert_at(Node::branch_with_param(branch), key, value);
                }
                //extension all match, insert to next
                if next_match_index == old_key.len() {
                    old_node.next = Self::insert_at(next, key.slice_from(next_match_index), value);
                    return Node::Extension(old_extension.clone());
                }
                //extension partial match, insert to next
                let new_next = Self::insert_at(
                    Node::extension(old_key.slice_from(next_match_index), next),
                    key.slice_from(next_match_index),
                    value,
                );
                old_node.nibble = old_key.sub_slice(0, next_match_index);
                old_node.next = new_next;

                Node::Extension(old_extension.clone())
            }
            Node::Branch(old_branch) => {
                let mut old_node = old_branch.borrow_mut();

                if key.value_at(0) == LEAF_FLAG {
                    old_node.value = Some(value);
                    return Node::Branch(old_branch.clone());
                }
                let old_child = old_node.child[key.value_at(0)].clone();
                let new_child = Self::insert_at(old_child, key.slice_from(1), value);
                old_node.child[key.value_at(0)] = new_child;
                Node::Branch(old_branch.clone())
            }
        }
    }

    fn get_at(node: &Node, key: &Nibble) -> Option<Vec<u8>> {
        match node {
            Node::Empty => None,
            Node::Leaf(leaf_node) => {
                let node = leaf_node.borrow();
                let key_end = &node.nibble;
                let value = node.value.clone();
                let next_match_index = key.match_len(key_end);
                if next_match_index == 0 {
                    return None;
                }
                if next_match_index != key.len() {
                    return None;
                }
                Some(value)
            }
            Node::Branch(branch_node) => {
                let node = branch_node.borrow();
                if key.is_empty() || key.value_at(0) == LEAF_FLAG {
                    node.value.clone()
                } else {
                    Self::get_at(&node.child[key.value_at(0)], &key.slice_from(1))
                }
            }
            Node::Extension(extension_node) => {
                let node = extension_node.borrow();
                let prefix = &node.nibble;
                let next = &node.next;
                let next_match_index = key.match_len(prefix);
                if next_match_index == 0 {
                    return None;
                }
                if next_match_index != prefix.len() {
                    return None;
                }
                Self::get_at(next, &key.slice_from(next_match_index))
            }
        }
    }

    fn delete_at(node: Node, key: &Nibble) -> (Node, bool) {
        match node {
            Node::Empty => (Node::Empty, false),
            Node::Leaf(leaf_node) => {
                let node = leaf_node.borrow_mut();
                let key_end = &node.nibble;
                (Node::Leaf(leaf_node.clone()), key_end == key)
            }
            Node::Branch(branch_node) => {
                let mut node = branch_node.borrow_mut();
                if key.value_at(0) == LEAF_FLAG {
                    return (Node::Branch(branch_node.clone()), true);
                }
                let (new_node, deleted) =
                    Self::delete_at(node.child[key.value_at(0)].clone(), &key.slice_from(1));
                if deleted {
                    node.insert_at(key.value_at(0), new_node);
                }
                (Node::Branch(branch_node.clone()), deleted)
            }
            Node::Extension(extension_node) => {
                let mut node = extension_node.borrow_mut();
                let prefix = &node.nibble;
                let next = node.next.clone();
                let next_match_index = key.match_len(prefix);
                if next_match_index == 0 || next_match_index != prefix.len() {
                    return (Node::Extension(extension_node.clone()), false);
                }
                let (new_sub, deleted) = Self::delete_at(next, &key.slice_from(next_match_index));
                if deleted {
                    node.next = new_sub;
                }
                (Node::Extension(extension_node.clone()), deleted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::trie::{PatriciaTrie, Trie};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_trie_insert() {
        let mut trie = PatriciaTrie::new();
        trie.insert(b"test".to_vec(), b"test".to_vec());
        trie.insert(b"test1".to_vec(), b"test1".to_vec());
        trie.insert(b"testdas".to_vec(), b"testdas".to_vec());
        assert_eq!(trie.get(b"test"), Some(b"test".to_vec()));
        assert_eq!(trie.get(b"test1"), Some(b"test1".to_vec()));
        assert_eq!(trie.get(b"testdas"), Some(b"testdas".to_vec()));
        println!("{:?}", trie.root);
    }

    #[test]
    fn test_trie_random_insert() {
        let mut trie = PatriciaTrie::new();
        for _ in 0..10000 {
            let rand_str: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
            let val = rand_str.as_bytes();
            trie.insert(val.to_vec(), val.to_vec());

            let v = trie.get(val);
            assert_eq!(v, Some(val.to_vec()));
        }
    }

    #[test]
    fn test_trie_remove() {
        let mut trie = PatriciaTrie::new();
        trie.insert(b"test".to_vec(), b"test".to_vec());
        let removed = trie.remove(b"test");
        assert!(removed)
    }

    #[test]
    fn test_trie_random_remove() {
        let mut trie = PatriciaTrie::new();

        for _ in 0..1000 {
            let rand_str: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
            let val = rand_str.as_bytes();
            trie.insert(val.to_vec(), val.to_vec());

            let removed = trie.remove(val);
            assert!(removed);
        }
    }

    #[test]
    fn insert_full_branch() {
        let mut trie = PatriciaTrie::new();

        trie.insert(b"test".to_vec(), b"test".to_vec());
        trie.insert(b"test1".to_vec(), b"test".to_vec());
        trie.insert(b"test2".to_vec(), b"test".to_vec());
        trie.insert(b"test23".to_vec(), b"test".to_vec());
        trie.insert(b"test33".to_vec(), b"test".to_vec());
        trie.insert(b"test44".to_vec(), b"test".to_vec());

        let v = trie.get(b"test");
        assert_eq!(Some(b"test".to_vec()), v);
    }
}
