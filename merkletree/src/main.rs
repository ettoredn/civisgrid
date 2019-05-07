// Owned children implementation requires only shared read-only ownership of children.
// Adding a parent reference to each node appears to require shared mutable ownership (RefCell).

// https://github.com/hyperledger/sawtooth-core/blob/master/validator/src/state/merkle.rs
// Hash trees allow _efficient and secure verification_ of the contents of large data structures
use sha3::{Sha3_256, Digest};
use std::fmt::Write;

// 'owned children' implementation without parent reference.
#[derive(Debug, Clone)]
enum Node<'a> {
    Leaf {
        data: &'a [u8],
        hash: String,
        // parent: &'a Option<Weak<Node<'a>>>,
    },
    Branch {
        hash: String,
        left: Box<Node<'a>>, // heap allocated _owned_ data. Workaround for size determinacy.
        right: Box<Node<'a>>, // heap allocated _owned_ data. Workaround for size determinacy.
        // parent: &'a Option<Weak<Node<'a>>>,
    }
}

/**
 * Balanced binary Merkle tree.
 * Balanced: left and right subtrees of every node differ in height by no more than 1.
 */
#[derive(Debug, Clone)]
struct MerkleTree<'a> {
    root: Node<'a>,
}
impl<'a> MerkleTree<'a> {

    /**
     * Time complexity wrt data items: n + log2( n/2 )
     */
    pub fn from_data(data: &[&'a[u8]]) -> MerkleTree<'a> {
        // Stores branches for the 'current' tree level to be processed
        let mut nodes = Vec::new();

        // The balanced binary Merkle tree is built starting from the leaves.
        for d/*: &&[u8] */ in data {
            let leaf = MerkleTree::make_leaf(d);
            nodes.push(leaf);
        }

        // process current level nodes to build nodes for the upper level.
        while nodes.len() > 1 {
            let mut parents: Vec<Node> = Vec::new(); // New nodes for the upper level

            for nodes/*: &[Node, Node]*/ in nodes.chunks(2) {
                // hash is the concatenation of leaves hashes
                let mut label = String::new();
                let label = nodes.iter().fold(&mut label, |label, node| {
                    let hash = match node {
                        Node::Branch{ hash, .. } => hash, // should not move because ll: &Node
                        Node::Leaf{ hash, .. } => hash,
                    };
                    label.push_str(hash);
                    label
                });

                let hash = sha3_hex(label.as_bytes());

                match nodes.len() {
                    2 => {
                        let b = MerkleTree::make_branch(hash, nodes[0].clone(), nodes[1].clone());
                        parents.push(b);
                    },
                    _ => parents.push(nodes[0].clone())
                }
            }

            nodes = parents; // parents is not dropped here because its ownership is moved
        }

        assert_eq!(nodes.len(), 1);
        let tree = MerkleTree{
            root: nodes.last().unwrap_or_else(|| panic!("Missing tree root !?")).clone()
        };

        tree
    }

    fn make_leaf(data: &[u8]) -> Node { // 1st + 2nd lifetime elision rule???
        Node::Leaf {
            hash: sha3_hex(data),
            data: data,
        }
    }

    fn make_branch(hash: String, child_left: Node<'a>, child_right: Node<'a>) -> Node<'a> {
        Node::Branch{
            hash: hash,
            left: Box::new(child_left),
            right: Box::new(child_right)
        }
    }
}

fn main() -> () {
    // let mut b = Box::new(4); // allocated on the heapz
    // *b = 123; // Deref trait
    // dbg!(b);

    let mut data: Vec<Vec<u8>> = Vec::new();
    for d/*: u8 */ in 1..=5u8 {
        // data.push(format!("Id = {}", d).into_bytes());
        data.push(vec![d]);
    }
    let refs: Vec<&[u8]> = data.iter().map(|d| d.as_slice()).collect();

    let tree = MerkleTree::from_data(refs.as_slice());
    dbg!(tree);
}

// #[derive(Debug, Clone, Eq)]
// struct Node {
//     data: Vec<u8>,
//     hash: String,
//     left: Node,
//     right: Node
// }
// impl Node {
//     pub fn new(data: Vec<u8>) -> Node {
//         // Takes ownership of data
//         Node {
//             hash: sha3_hex(&data.as_slice()),
//             data: data,
//         }
//     }

//     pub fn set_data(&mut self, data: Vec<u8>) {
//         self.data = data;
//         self.hash = sha3_hex(self.data.as_slice());
//     }
// }

fn sha3_hex(data: &[u8]) -> String {
    let hash = Sha3_256::digest(data);
    let mut hex = String::new();

    for byte in hash {
        write!(hex, "{:02x}", byte).unwrap();
    }

    hex
}

// #[derive(Default, Debug, PartialEq, Clone)]
// struct Node {
//     value: Option<Vec<u8>>,
//     children: BTreeMap<String, String>,
// }


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

    #[test]
    fn it_works() {
        assert_eq!(2+2, 5, "2+2 != 4")
    }

    #[test]
    fn sha3_hash() {
        assert_eq!(sha3_hex("Some random data".as_bytes()), "5b054cb1c47ebc3e0bd156e474a36ab2068807eb14bbe609639fc1f9bf53261a")
    }
}