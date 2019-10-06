use std::fmt;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use sha3::{Sha3_256, Digest};
use faster_hex::{hex_string};

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
    label: String,
    parent: RefCell<Weak<Node<'a>>>, // interior mutability
}
impl<'a> Hash for Node<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.label.as_bytes() {
            state.write_u8(*b);
        }
    }
}

/**
 * Balanced binary Merkle tree.
 * Balanced: left and right subtrees of every node differ in height by no more than 1.
 */
pub struct MerkleTree<'a> {
    root: Rc<Node<'a>>,
    nodes: HashMap<String, Rc<Node<'a>>>, // indexed by label
    // data_map: HashMap<String, Rc<Node<'a>>>,
}
impl<'a> fmt::Debug for MerkleTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.root)
    }
}

impl<'a> MerkleTree<'a> {
    /**
     * Time complexity wrt data items: n + log2(n)
     * 
     * NEW ALGO TODO for proper+complete binary tree
     * - let complete_tree = make_tree(leaves.rev()[0..floor(log2(leaves.length))])
     *     ^--> create a proper and complete tree starting from the _last_ "smallest power of 2" leaves
     * - add new level s.t. 
     * 
     */
    pub fn from_data(data: &[&'a[u8]]) -> MerkleTree<'a> {
        // Stores branches for the 'current' tree level to be processed
        let mut nodes_stack: Vec<Rc<Node<'a>>> = vec![];
        let mut nodes_map: HashMap<String, Rc<Node<'a>>> = HashMap::new();
        // let mut leaves_map = HashMap::new();

        // The balanced binary Merkle tree is built starting from the leaves.
        for d/*: &[u8] */ in data.into_iter().rev() {
            let _z: &'a[u8] = d;

            let leaf = MerkleTree::make_leaf(d);
            nodes_map.insert(leaf.label.clone(), leaf.clone());
            nodes_stack.push(leaf);
        }

        // process current level nodes to build nodes for the upper level.
        while nodes_stack.len() > 1 {
            let mut parents: Vec<Rc<Node<'a>>> = Vec::new(); // New nodes for the upper level

            // Builds parent nodes
            while nodes_stack.len() > 1 {
                let child_left = nodes_stack.pop().unwrap();
                let child_right = nodes_stack.pop().unwrap(); // nodes.len() > 1

                let b = MerkleTree::make_branch(child_left, child_right); // moves ownership of children
                nodes_map.insert(b.label.clone(), b.clone());
                parents.insert(0, b);
            }

            if let Some(n) = nodes_stack.pop() {
                parents.insert(0, n); // odd number of nodes, moves the remaining node to the right of the tree
            }

            nodes_stack = parents;
        }

        assert_eq!(nodes_stack.len(), 1);
        let tree = MerkleTree{
            root: nodes_stack.pop().unwrap_or_else(|| panic!("Empty merkle tree!?")).clone(),
            nodes: nodes_map
        };

        tree
    }

    // #[allow(dead_code)]
    // pub fn complete_from_data(data: &[&'a[u8]]) -> MerkleTree<'a> {

    // }

    /**
     * [Auth0, Auth1, .., Auth(i)] where 0 <= i < Height
     * i.e. from the leaf's sibling to the root's children
     */
    pub fn make_proof(&self, data: &'a [u8]) -> Result<Vec<String>, &'static str> {
        let hash = MerkleTree::sha3_hex(data);

        // let node = *self..
        let leaf: &Rc<Node> = self.nodes.get(hash.as_str()).ok_or_else(|| "Provided data not included in the tree")?;

        let mut node = leaf.clone();
        let mut proof = Vec::new();
        while let Some(parent) = node.clone().parent.borrow().upgrade() {
            match &parent.r#type { // parent: Rc<Node>
                NodeType::Branch{left: l, right: r} => {
                    if Rc::ptr_eq(l, &node) {
                        proof.push(r.label.clone());
                    } else if Rc::ptr_eq(r, &node) {
                        proof.push(l.label.clone());
                    } else {
                        panic!("Wrong parent<->children references");
                    }
                }
                _ => ()
            }

            node = parent;
        }

        // assert proof length = height - 1
        return Ok(proof);
    }

    /**
     * Problem:
     * the current method requires knowledge of the tree structure
     * in order to properly concatenate hashes (left||right !== right||left)s
     */
    pub fn authenticate(&self, data: &'a [u8], proof: &[String]) -> bool {
        let mut node = match self.nodes.get(&MerkleTree::make_label(data)) {
            Some(n) => n.clone(),
            None => return false
        };

        for auth in proof {
            let parent = match node.parent.borrow().upgrade() {
                Some(n) => n,
                None => return false // every proof's hash must have a parent. Root is not included in proofs.
            };

            let sibling: &Rc<Node<'a>> = match &parent.r#type {
                NodeType::Branch{left: l, right: r} => {
                    match &l.label == auth {
                        true => l,
                        false => r,
                    }
                }
                _ => panic!("Parent must have children!")
            };

            if sibling.label != *auth {
                return false;
            }

            node = parent;
        }

        true
    }

    fn make_leaf(data: &'a [u8]) -> Rc<Node<'a>> { // 1st + 2nd lifetime elision rule???
        Rc::new(Node{
            r#type: NodeType::Leaf{ data: data },
            label: MerkleTree::make_label(data),
            parent: RefCell::new(Weak::new()),
        })
    }

    fn make_branch(child_left: Rc<Node<'a>>, child_right: Rc<Node<'a>>) -> Rc<Node<'a>> {
        let label: String = MerkleTree::make_label([
            child_left.label.as_str(), child_right.label.as_str()
        ].concat().as_bytes());

        // Rc needed to be able to create a weak reference for its children
        let branch = Rc::new(Node{
            r#type: NodeType::Branch {
                left: child_left,
                right: child_right,
            },
            label: label,
            parent: RefCell::new(Weak::new()),
        });

        if let NodeType::Branch{left: l, right: r, ..} = &branch.r#type {
            *l.parent.borrow_mut() = Rc::downgrade(&branch);
            *r.parent.borrow_mut() = Rc::downgrade(&branch);
        }

        branch
    }

    #[allow(dead_code)]
    fn node_depth(&self, node: &Rc<Node>) -> usize {
        let mut depth = 0;
        let mut node: Rc<Node> = node.clone();

        while let Some(parent) = node.clone().parent.borrow().upgrade() {
            node = parent;
            depth = depth + 1;
        }

        depth
    }

    #[allow(dead_code)]
    fn count_nodes(&self) -> usize {
        MerkleTree::count_descendants(&self.root)
    }

    #[allow(dead_code)]
    fn count_leaves(&self) -> usize {
        MerkleTree::count_descendant_leaves(&self.root)
    }

    fn count_descendants(node: &Rc<Node>) -> usize {
        1 + match &node.r#type {
            NodeType::Branch{left: l, right: r, ..} => MerkleTree::count_descendants(l) + MerkleTree::count_descendants(r),
            NodeType::Leaf{..} => 0
        }
    }

    fn count_descendant_leaves(node: &Rc<Node>) -> usize {
        match &node.r#type {
            NodeType::Branch{left: l, right: r, ..} => MerkleTree::count_descendant_leaves(l) + MerkleTree::count_descendant_leaves(r),
            NodeType::Leaf{..} => 1,
        }
    }

    fn make_label(data: &[u8]) -> String {
        MerkleTree::sha3_hex(data)
    }

    fn sha3_hex(data: &[u8]) -> String {
        hex_string(Sha3_256::digest(data).as_slice()).unwrap()
    }
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
    use rand::{thread_rng, Rng};

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
        let data = "Some random data".as_bytes();
        assert_eq!(hex_string(Sha3_256::digest(data).as_slice()).unwrap(), "5b054cb1c47ebc3e0bd156e474a36ab2068807eb14bbe609639fc1f9bf53261a");
    }

    #[test]
    #[ignore]
    fn tree_nodes_count() {
        for leaves_count in 2..=97 {
            let data: Vec<Vec<u8>> = make_data(leaves_count);
            let data_refs: Vec<&[u8]> = make_data_refs(&data);

            let tree = MerkleTree::from_data(&data_refs);
            let leaves = tree.count_leaves();
            let branches = tree.count_nodes() - leaves;
            println!("Merkle tree leaves={} branches={} leaves-branches={}", leaves, branches, leaves-branches);
            assert_eq!(branches, leaves - 1);
        }
    }

    #[test]
    fn tree_leaf_depth() {
        let data: Vec<Vec<u8>> = make_data(11);
        let data_refs: Vec<&[u8]> = make_data_refs(&data);
        let tree = MerkleTree::from_data(&data_refs);

        let rightmost = dbg!(tree.nodes.get("8bf02b8b238233453488311be9b316e58ab7e1356ce948cb90dfef1af56992eb").unwrap()); //9
        let leftmost = dbg!(tree.nodes.get("2767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7").unwrap()); //1
        let center = dbg!(tree.nodes.get("989216075a288af2c12f115557518d248f93c434965513f5f739df8c9d6e1932").unwrap()); // 4
        dbg!(tree.node_depth(rightmost));
        dbg!(tree.node_depth(leftmost));
        dbg!(tree.node_depth(center));

        assert_eq!(tree.node_depth(&tree.root), 0);

        match &tree.root.r#type {
            // root
            NodeType::Branch{left: l, right: r} => {
                assert_eq!(tree.node_depth(r), 1);
                assert_eq!(tree.node_depth(l), 1);

                match &l.r#type {
                    // root -> left
                    NodeType::Branch{left: l_l, right: l_r, ..} => {
                        assert_eq!(tree.node_depth(l_l), 2);
                        assert_eq!(tree.node_depth(l_r), 2);

                        match &l_l.r#type {
                            // root -> left -> left
                            NodeType::Branch{left: l_l_l, right: l_l_r, .. } => {
                                assert_eq!(tree.node_depth(l_l_l), 3);
                                assert_eq!(tree.node_depth(l_l_r), 3);
                            },
                            _ => panic!("Unexpected node type"),
                        }
                    }
                    _ => panic!("Unexpected node type"),
                }

                match &r.r#type {
                    // root -> right
                    NodeType::Branch{left: r_l, right: r_r} => {
                        assert_eq!(tree.node_depth(r_l), 2);
                        assert_eq!(tree.node_depth(r_r), 2);

                        match &r_r.r#type {
                            // root -> right -> right
                            NodeType::Leaf{..} => assert_eq!(tree.node_depth(&r_r), 2),
                            _ => panic!("Unexpected root type NodeType::Leaf"),
                        }
                    },
                    _ => panic!("Unexpected node type"),
                }

            },
            _ => panic!("Unexpected root type NodeType::Leaf"),
        };
    }

    #[test]
    fn data_proofs() {
        let data: Vec<Vec<u8>> = make_data(5);
        let data_refs: Vec<&[u8]> = make_data_refs(&data);
        let tree = MerkleTree::from_data(&data_refs);

        let datum: &[u8] = data_refs[4];
        let proof = tree.make_proof(datum).unwrap();
        assert_eq!(proof, [
            "5ea0ffd548dacbbfc23452a271a0fab46f39114bda991ce85bf0386ca2294d3f"
        ]);
        assert!(tree.authenticate(datum, &proof));

        let datum: &[u8] = data_refs[3];
        let proof = tree.make_proof(datum).unwrap();
        assert_eq!(proof, [
            "e3ed56bd086d8958483a12734fa0ae7f5c8bb160ef9092c67e82ed9b19e4c7b2",
            "bfec02f100e0803e2124e5c28a567ccc5547640e96aa1ca3ed8798ba21d2e1ab",
            "3b0c4d506212cd7e7b88bc93b5b1811ab5de6796d2780e9de7378c87fe9a80a6"
        ]);
        assert!(tree.authenticate(datum, &proof));

        let datum: &[u8] = data_refs[thread_rng().gen_range(0u8, 5u8) as usize];
        let proof = tree.make_proof(datum).unwrap();
        assert!(tree.authenticate(datum, &proof), format!("Invalid proof for {:?}", datum));

        assert!(tree.make_proof(&[6u8]).is_err());
    }

    #[test]
    fn single_node_tree() {
        let data = make_data(1);
        let refs = make_data_refs(&data);
        let tree = MerkleTree::from_data(&refs);

        assert_eq!(tree.count_leaves(), 1);
        assert_eq!(tree.count_nodes(), 1);
        assert_eq!(tree.node_depth(&tree.root), 0);

        assert!(tree.make_proof(refs[0]).unwrap().is_empty());
        assert!(tree.authenticate(refs[0], &tree.make_proof(refs[0]).unwrap()));
    }

    #[test]
    fn complete_tree() {
        // Complete (=> balanced, but not perfect) proper binary tree
        for ln in 1..1000 {
            // Levels start from 0 i.e. = depth
            let last_filled_level = (ln as f64).log2().floor() as u32;
            let leaves_on_last_level = (ln - 2usize.pow(last_filled_level)) * 2; // multiple of 2

            println!("Leaves={}, last_filled_level={}, leaves_last_level={}", ln, last_filled_level, leaves_on_last_level);
            assert!(leaves_on_last_level < 2usize.pow(last_filled_level+1));

            assert_eq!(leaves_on_last_level % 2, 0);
            // put first <leaves_on_last_level> leaves on the stack in rev order   | O(L-1)
            // while (take 2 nodes from the stack):
            //   make parent branch;                                               | O(k)
            //   push branch on TOP of the stack;                                  | O(2)
        }
    }
}