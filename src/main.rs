mod merkletree;

// Refactor to mod merkletree{MerkleTree, Proof}
// Owned children implementation requires only shared read-only ownership of children.
// Adding a parent reference to each node appears to require shared mutable ownership (RefCell).

// https://github.com/hyperledger/sawtooth-core/blob/master/validator/src/state/merkle.rs
// Hash trees allow _efficient and secure verification_ of the contents of large data structures

use civisgrid::import_me;

fn main() -> () {
    // let mut b = Box::new(4); // allocated on the heapz
    // *b = 123; // Deref trait
    // dbg!(b);

    println!("+++ CIVISgrid +++");

    import_me();

    let mut data: Vec<Vec<u8>> = Vec::new();
    for d in 1..=9 {
        // data.push(format!("Id = {}", d).into_bytes());
        data.push(vec![d as u8]);
    }
    let refs: Vec<&[u8]> = data.iter().map(|d| d.as_slice()).collect();

    let tree = merkletree::MerkleTree::from_data(&refs);
    dbg!(&tree);
    let proof = tree.make_proof(refs[5]).unwrap();
    assert!(tree.authenticate(refs[5], &proof));
}
