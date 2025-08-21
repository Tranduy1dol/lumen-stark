use std::marker::PhantomData;

use digest::{Digest as HasherTrait, Output};

use crate::common::bytes::ToBytes;

pub type Path<H> = Vec<Output<H>>;

#[derive(Clone, Debug, Default)]
pub struct MerkleTree<H: HasherTrait> {
    _phantom: PhantomData<H>,
}

impl<H> MerkleTree<H>
where
    H: HasherTrait,
{
    fn hash_pair(left: &Output<H>, right: &Output<H>) -> Output<H> {
        let mut hasher = H::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize()
    }

    fn hash_data<T: ToBytes>(data_array: &[T]) -> Vec<Output<H>> {
        data_array
            .iter()
            .map(|item| H::digest(item.to_bytes()))
            .collect()
    }

    fn commit_inner(leafs: &[Output<H>]) -> Output<H> {
        assert!(
            leafs.len().is_power_of_two(),
            "Merkle tree requires the number of leaves to be a power of two."
        );

        if leafs.len() == 1 {
            return leafs[0].clone();
        }

        let (left_half, right_half) = leafs.split_at(leafs.len() / 2);
        let left_hash = Self::commit_inner(left_half);
        let right_hash = Self::commit_inner(right_half);

        Self::hash_pair(&left_hash, &right_hash)
    }

    fn open_inner(index: usize, leafs: &[Output<H>]) -> Path<H> {
        assert!(
            leafs.len().is_power_of_two(),
            "Merkle tree requires the number of leaves to be a power of two."
        );
        assert!(
            index < leafs.len(),
            "Cannot open proof for an invalid index."
        );

        if leafs.len() == 2 {
            return vec![leafs[1 - index].clone()];
        }

        let mid = leafs.len() / 2;
        if index < mid {
            let mut path = Self::open_inner(index, &leafs[..mid]);
            path.push(leafs[mid].clone());
            path
        } else {
            let mut path = Self::open_inner(index - mid, &leafs[mid..]);
            path.push(leafs[mid].clone());
            path
        }
    }

    fn verify_inner(root: &Output<H>, mut index: usize, leaf: &Output<H>, path: &Path<H>) -> bool {
        assert!(
            index < (1 << path.len()),
            "Cannot verify proof for an invalid index."
        );

        let mut current_hash = leaf.clone();
        for sibling_hash in path {
            if index % 2 == 0 {
                current_hash = Self::hash_pair(&current_hash, sibling_hash);
            } else {
                current_hash = Self::hash_pair(sibling_hash, &current_hash);
            }
            index >>= 1;
        }

        current_hash == *root
    }

    pub fn commit<T: ToBytes>(data_array: &[T]) -> Output<H> {
        let leafs = Self::hash_data(data_array);
        Self::commit_inner(&leafs)
    }

    pub fn open<T: ToBytes>(index: usize, data_array: &[T]) -> Path<H> {
        let leafs = Self::hash_data(data_array);
        Self::open_inner(index, &leafs)
    }

    pub fn verify<T: ToBytes>(
        root: &Output<H>,
        index: usize,
        data_element: &T,
        path: &Path<H>,
    ) -> bool {
        let leafs_hash = H::digest(data_element.to_bytes());
        Self::verify_inner(root, index, &leafs_hash, path)
    }
}
