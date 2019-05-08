use sha3::{Sha3_256, Digest};
use std::fmt::Write;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug, Clone)]
enum NodeType<'a> {
    Branch{ 
        left: Rc<Node<'a>>, // shared ownership
        right: Rc<Node<'a>>, // shared ownership
    },
    Leaf{ 
        data: &'a [u8]
    },
}
#[derive(Debug)]
struct Node<'a> {
    r#type: NodeType<'a>,
    hash: String,
    parent: RefCell<Weak<Node<'a>>>, // single ownership
}

/**
 * Balanced binary Merkle tree.
 * Balanced: left and right subtrees of every node differ in height by no more than 1.
 */
#[derive(Debug)]
pub struct MerkleTree<'a> {
    root: Rc<Node<'a>>,
}
impl<'a> MerkleTree<'a> {
    /**
     * Time complexity wrt data items: n + log2(n)
     */
    pub fn from_data(data: &[&'a[u8]]) -> MerkleTree<'a> {
        // Stores branches for the 'current' tree level to be processed
        let mut nodes: Vec<Rc<Node<'a>>> = vec![];

        // The balanced binary Merkle tree is built starting from the leaves.
        for d/*: &[u8] */ in data.into_iter().rev() {
            let _z: &'a[u8] = d;

            let leaf = MerkleTree::make_leaf(d);
            nodes.push(leaf);
        }

        // process current level nodes to build nodes for the upper level.
        while nodes.len() > 1 {
            let mut parents: Vec<Rc<Node<'a>>> = Vec::new(); // New nodes for the upper level

            // Builds parent nodes
            while nodes.len() > 1 {
                let child_left = nodes.pop().unwrap();
                let child_right = nodes.pop().unwrap(); // nodes.len() > 1

                let label: String = [child_left.hash.as_str(), child_right.hash.as_str()].concat();
                let hash = sha3_hex(label.as_bytes());

                let b = MerkleTree::make_branch(hash, child_left, child_right); // moves ownership of children
                parents.insert(0, b);
            }

            if let Some(n) = nodes.pop() {
                parents.insert(0, n); // odd number of nodes, moves the remaining node to the right of the tree
            }

            nodes = parents;
        }

        assert_eq!(nodes.len(), 1);
        let tree = MerkleTree{
            root: nodes.pop().unwrap_or_else(|| panic!("Missing tree root !?")).clone()
        };

        tree
    }

    fn make_leaf(data: &'a [u8]) -> Rc<Node<'a>> { // 1st + 2nd lifetime elision rule???
        Rc::new(Node{
            r#type: NodeType::Leaf{ data: data },
            hash: sha3_hex(data),
            parent: RefCell::new(Weak::new()),
        })
    }

    fn make_branch(hash: String, child_left: Rc<Node<'a>>, child_right: Rc<Node<'a>>) -> Rc<Node<'a>> {
        assert_eq!(Rc::strong_count(&child_left), 1);
        assert_eq!(Rc::strong_count(&child_right), 1);

        // Rc needed to be able to create a weak reference for its children
        let branch = Rc::new(Node{
            r#type: NodeType::Branch {
                left: child_left,
                right: child_right,
            },
            hash: hash,
            parent: RefCell::new(Weak::new()),
        });

        if let NodeType::Branch{left: l, right: r, ..} = &branch.r#type {
            *l.parent.borrow_mut() = Rc::downgrade(&branch);
            *r.parent.borrow_mut() = Rc::downgrade(&branch);
        }

        branch
    }

    #[allow(dead_code)]
    pub fn count_nodes(&self) -> usize {
        MerkleTree::count_descendants(&self.root)
    }

    #[allow(dead_code)]
    pub fn count_leaves(&self) -> usize {
        MerkleTree::count_descendant_leaves(&self.root)
    }

    #[allow(dead_code)]
    fn count_descendants(node: &Rc<Node>) -> usize {
        1 + match &node.r#type {
            NodeType::Branch{left: l, right: r, ..} => MerkleTree::count_descendants(l) + MerkleTree::count_descendants(r),
            NodeType::Leaf{..} => 0
        }
    }

    #[allow(dead_code)]
    fn count_descendant_leaves(node: &Rc<Node>) -> usize {
        match &node.r#type {
            NodeType::Branch{left: l, right: r, ..} => MerkleTree::count_descendant_leaves(l) + MerkleTree::count_descendant_leaves(r),
            NodeType::Leaf{..} => 1,
        }
    }

    // TODO
    // fn get_auth_path(leaf: &Node) {
    //     // I would like to have the parent!!!!
    // }

    // fn authenticate(leaf: &Node, path: &[Node]) {

    // }
}

fn sha3_hex(data: &[u8]) -> String {
    let hash = Sha3_256::digest(data);
    let mut hex = String::new();

    for byte in hash {
        write!(hex, "{:02x}", byte).unwrap();
    }

    hex
}

// Merkle Root represent a version of the state
// get(root, addr) should return data stored at address, given a specific version of the state

// Sawtooth uses the Radix Merkle Trie to make fast queries to the state and to guarantee its integrity
// and transaction order
// Sawtooth uses the Radix Merkle tree to retrieve the location of data???
// .. but if LMDB is already a key/value store, why cant we just get(address) ?
// .. because for whatever reason an address

#[cfg(test)]
mod tests {
    use super::*; // includes private functions

    fn make_data(amount: usize) -> Vec<Vec<u8>> {
        let mut data: Vec<Vec<u8>> = Vec::new();
        for d in 1..=amount {
            // data.push(format!("Id = {}", d).into_bytes());
            data.push(vec![d as u8]);
        }

        data
    }

    fn make_data_refs<'a>(data: &'a Vec<Vec<u8>>) -> Vec<&'a [u8]> {
        let refs: Vec<&[u8]> = data.iter().map(|d| d.as_slice()).collect();

        refs
    }

    #[test]
    fn sha3_hash() {
        assert_eq!(sha3_hex("Some random data".as_bytes()), "5b054cb1c47ebc3e0bd156e474a36ab2068807eb14bbe609639fc1f9bf53261a")
    }

    #[test]
    fn tree_nodes_count() {
        let data: Vec<Vec<u8>> = make_data(256);
        let data_refs: Vec<&[u8]> = make_data_refs(&data);

        let tree = MerkleTree::from_data(&data_refs);

        assert_eq!(tree.count_nodes(), 256+255);
        assert_eq!(tree.count_leaves(), data.len());
    }
}