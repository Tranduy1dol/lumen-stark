use serde::{Serialize, de::DeserializeOwned};

pub trait TranscriptSerializer {
    type Error: std::fmt::Debug;

    fn serialize<T: Serialize>(obj: &T) -> Result<Vec<u8>, Self::Error>;

    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Self::Error>;
}
