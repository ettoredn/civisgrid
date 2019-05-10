mod merkletree;

// Refactor to mod merkletree{MerkleTree, Proof}
// Owned children implementation requires only shared read-only ownership of children.
// Adding a parent reference to each node appears to require shared mutable ownership (RefCell).

// https://github.com/hyperledger/sawtooth-core/blob/master/validator/src/state/merkle.rs
// Hash trees allow _efficient and secure verification_ of the contents of large data structures

fn main() -> () {
    // let mut b = Box::new(4); // allocated on the heapz
    // *b = 123; // Deref trait
    // dbg!(b);

    println!("+++ CIVISgrid +++");

    let mut data: Vec<Vec<u8>> = Vec::new();
    for d in 1..=8 {
        // data.push(format!("Id = {}", d).into_bytes());
        data.push(vec![d as u8]);
    }
    let refs: Vec<&[u8]> = data.iter().map(|d| d.as_slice()).collect();

    let tree = merkletree::MerkleTree::from_data(&refs);
    let _proof = tree.make_proof(&[5u8]);

    // dbg!(&tree);
    // let leaves = tree.count_leaves();
    // let branches = tree.count_nodes() - leaves;
    // println!("Merkle tree leaves={} branches={}", leaves, branches);
}
