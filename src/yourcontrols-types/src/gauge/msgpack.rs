use rmp_serde::{decode::from_read_ref, encode::to_vec};
use serde::{de::DeserializeOwned, Serialize};

use crate::{FragmentedMessage, MessageFragmenter};

pub struct MessagePackFragmenter {
    fragmenter: MessageFragmenter,
}

impl MessagePackFragmenter {
    pub fn new(fragment_size: usize) -> Self {
        Self {
            fragmenter: MessageFragmenter::new(fragment_size),
        }
    }

    pub fn process_fragment_bytes<T>(&mut self, data: &[u8]) -> Result<T, rmp_serde::decode::Error>
    where
        T: DeserializeOwned,
    {
        let fragment: FragmentedMessage = from_read_ref(data)?;

        let bytes = self
            .fragmenter
            .process_fragment(fragment)
            .ok_or_else(|| rmp_serde::decode::Error::OutOfRange)?;

        Ok(from_read_ref(&bytes)?)
    }

    pub fn into_fragmented_message_bytes<T>(
        &self,
        data: &T,
    ) -> Result<Vec<Vec<u8>>, rmp_serde::encode::Error>
    where
        T: Serialize,
    {
        let bytes = to_vec(data)?;
        let fragmented_messages = self.fragmenter.fragment_bytes(bytes);

        return Ok(fragmented_messages
            .into_iter()
            .map(|x| to_vec(&x).unwrap())
            .collect());
    }
}
