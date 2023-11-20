use crate::constant::LEAF_FLAG;
use crate::nibble::Nibble;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub enum Node {
    #[default]
    Empty,
    Leaf(Rc<RefCell<LeafNode>>),
    Branch(Rc<RefCell<BranchNode>>),
    Extension(Rc<RefCell<ExtensionNode>>),
}

impl Node {
    pub fn leaf(nibble: Nibble, value: Vec<u8>) -> Self {
        Node::Leaf(Rc::new(RefCell::new(LeafNode { nibble, value })))
    }

    pub fn branch() -> Self {
        Node::Branch(Rc::new(RefCell::new(BranchNode {
            child: empty_children(),
            value: None,
        })))
    }

    pub fn branch_with_param(node: BranchNode) -> Self {
        Node::Branch(Rc::new(RefCell::new(node)))
    }

    pub fn extension(nibble: Nibble, next: Node) -> Self {
        Node::Extension(Rc::new(RefCell::new(ExtensionNode { nibble, next })))
    }
}

#[derive(Debug, Default)]
pub struct LeafNode {
    pub nibble: Nibble,
    pub value: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct BranchNode {
    pub child: [Node; 16],
    pub value: Option<Vec<u8>>,
}

impl BranchNode {
    pub fn new() -> Self {
        BranchNode {
            child: empty_children(),
            value: None,
        }
    }
    pub fn insert_at(&mut self, index: usize, node: Node) {
        match index {
            index if index > LEAF_FLAG => panic!("insert index not valid!"),
            index if index == LEAF_FLAG => {
                if let Node::Leaf(leaf) = node {
                    self.value = Some(leaf.borrow().value.clone());
                } else {
                    panic!("The n must be leaf node")
                }
            }
            _ => self.child[index] = node,
        }
    }
}

#[derive(Debug, Default)]
pub struct ExtensionNode {
    pub nibble: Nibble,
    pub next: Node,
}

pub fn empty_children() -> [Node; 16] {
    [
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
    ]
}
