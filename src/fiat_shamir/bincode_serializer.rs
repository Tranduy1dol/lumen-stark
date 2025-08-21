use serde::{Serialize, de::DeserializeOwned};

use crate::common::transcript_serializer::TranscriptSerializer;

pub struct BincodeSerializer;

impl TranscriptSerializer for BincodeSerializer {
    type Error = bincode::Error;

    fn serialize<T: Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(obj)
    }

    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Self::Error> {
        bincode::deserialize(bytes)
    }
}
