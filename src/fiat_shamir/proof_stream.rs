use serde::{de::DeserializeOwned, Serialize};
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};

/// A ProofStream helps construct a non-interactive proof transcript by serializing
/// objects and using them to derive challenges via the Fiat-Shamir transform.
///
/// This is a Rust implementation that mirrors the functionality of the provided
/// Python `ProofStream` class. It uses `serde` for serialization (like Python's `pickle`)
/// and the `sha3` crate for SHAKE256 hashing.
///
/// The stream is generic over the type `T` of objects it can hold, requiring them
/// to be serializable and deserializable.
pub struct ProofStream<T: Serialize + DeserializeOwned> {
    pub objects: Vec<T>,
    pub read_index: usize,
}

impl<T: Serialize + DeserializeOwned> ProofStream<T> {
    /// Creates a new, empty `ProofStream`.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            read_index: 0,
        }
    }

    /// Pushes a new object onto the stream.
    pub fn push(&mut self, obj: T) {
        self.objects.push(obj);
    }

    /// Pulls the next object from the stream for the verifier to read.
    ///
    /// # Panics
    ///
    /// Panics if the read index is out of bounds (i.e., the stream is empty
    /// or all objects have been read).
    pub fn pull(&mut self) -> &T {
        assert!(
            self.read_index < self.objects.len(),
            "ProofStream: cannot pull object; queue empty."
        );
        let obj = &self.objects[self.read_index];
        self.read_index += 1;
        obj
    }

    /// Serializes all objects in the stream into a byte vector.
    ///
    /// This uses `bincode` for a compact binary representation.
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self.objects)
    }

    /// Deserializes a byte slice into a `ProofStream`.
    ///
    /// The `read_index` of the new stream is initialized to 0.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let objects: Vec<T> = bincode::deserialize(bytes)?;
        Ok(Self {
            objects,
            read_index: 0,
        })
    }

    /// Computes a challenge for the prover using the Fiat-Shamir transform.
    ///
    /// It serializes *all* objects currently in the stream, hashes them with
    /// SHAKE256, and returns the requested number of bytes.
    pub fn prover_fiat_shamir(&self, num_bytes: usize) -> Result<Vec<u8>, bincode::Error> {
        let mut hasher = Shake256::default();
        let serialized_objects = bincode::serialize(&self.objects)?;
        hasher.update(&serialized_objects);
        let mut reader = hasher.finalize_xof();
        let mut result = vec![0u8; num_bytes];
        reader.read(&mut result);
        Ok(result)
    }

    /// Computes a challenge for the verifier using the Fiat-Shamir transform.
    ///
    /// It serializes only the objects that have been read so far (up to `read_index`),
    /// hashes them with SHAKE256, and returns the requested number of bytes.
    pub fn verifier_fiat_shamir(&self, num_bytes: usize) -> Result<Vec<u8>, bincode::Error> {
        let mut hasher = Shake256::default();
        let serialized_objects = bincode::serialize(&self.objects[..self.read_index])?;
        hasher.update(&serialized_objects);
        let mut reader = hasher.finalize_xof();
        let mut result = vec![0u8; num_bytes];
        reader.read(&mut result);
        Ok(result)
    }
}

impl<T: Serialize + DeserializeOwned> Default for ProofStream<T> {
    fn default() -> Self {
        Self::new()
    }
}