use std::marker::PhantomData;

use serde::{Serialize, de::DeserializeOwned};
use sha3::{
    Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};

use crate::common::transcript_serializer::TranscriptSerializer;
use crate::fiat_shamir::bincode_serializer::BincodeSerializer;

pub struct ProofStream<T, H = Shake256, S = BincodeSerializer>
where
    T: Serialize + DeserializeOwned,
    H: ExtendableOutput + Update,
    S: TranscriptSerializer,
{
    pub objects: Vec<T>,
    pub read_index: usize,
    _phantom_h: PhantomData<H>,
    _phantom_s: PhantomData<S>,
}

impl<T, H, S> ProofStream<T, H, S>
where
    T: Serialize + DeserializeOwned,
    H: ExtendableOutput + Update,
    S: TranscriptSerializer,
{
    /// Creates a new, empty `ProofStream`.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            read_index: 0,
            _phantom_h: Default::default(),
            _phantom_s: Default::default(),
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
    /// The serialization format is determined by the `S` type parameter.
    pub fn serialize(&self) -> Result<Vec<u8>, S::Error> {
        S::serialize(&self.objects)
    }

    /// Deserializes a byte slice into a `ProofStream`.
    ///
    /// The `read_index` of the new stream is initialized to 0.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, S::Error> {
        let objects: Vec<T> = S::deserialize(bytes)?;
        Ok(Self {
            objects,
            read_index: 0,
            _phantom_h: Default::default(),
            _phantom_s: Default::default(),
        })
    }

    /// Computes a challenge for the prover using the Fiat-Shamir transform.
    ///
    /// It serializes *all* objects currently in the stream, hashes them with
    /// the chosen hash function `H`, and returns the requested number of bytes.
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
    /// hashes them with the chosen hash function `H`, and returns the requested number of bytes.
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

impl<T, H, S> Default for ProofStream<T, H, S>
where
    T: Serialize + DeserializeOwned,
    H: ExtendableOutput + Update,
    S: TranscriptSerializer,
{
    fn default() -> Self {
        Self::new()
    }
}
