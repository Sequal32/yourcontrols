use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Serves as a header for a FragmentedMessage.
/// Includes the index of the complete message that the fragment is a part of and the fragment count of the complete message.
///
/// The total header size is 12 bytes (aligned) or 10 bytes (packed)
#[derive(Serialize, Deserialize)]
pub struct MessageHeader {
    fragment_index: u8,
    fragment_count: u8,
    fragment_size: usize,
}

/// A fragmented message with a header, and the fragmented bytes.
#[derive(Serialize, Deserialize)]
pub struct FragmentedMessage {
    header: MessageHeader,
    bytes: Vec<u8>,
}

/// Fragments into FragmentedMessages and combines FragmentedMessages into a Vec<u8>.
pub struct MessageFragmenter {
    fragment_size: usize,
    processing_fragments: VecDeque<FragmentedMessage>,
}

impl MessageFragmenter {
    pub fn new(fragment_size: usize) -> Self {
        Self {
            fragment_size,
            processing_fragments: VecDeque::new(),
        }
    }

    /// Turns a Vec<u8> into FragmentedMessages.
    pub fn fragment_bytes(&self, bytes: Vec<u8>) -> Vec<FragmentedMessage> {
        let mut fragmented_messages = Vec::new();

        let fragment_count = (bytes.len() as f32 / self.fragment_size as f32).ceil() as u8;

        let mut bytes_iter = bytes.into_iter();

        for fragment_index in 0..fragment_count {
            let bytes: Vec<u8> = (&mut bytes_iter).take(self.fragment_size).collect();

            fragmented_messages.push(FragmentedMessage {
                header: MessageHeader {
                    fragment_index,
                    fragment_count,
                    fragment_size: bytes.len(),
                },
                bytes,
            })
        }

        fragmented_messages
    }

    /// Processes a fragment.
    /// If the fragment is part of a complete message, store the fragment to be combined later.
    /// If the fragment is the last piece of a complete message, combine all previously received fragments into a Vec<u8>.
    pub fn process_fragment(&mut self, fragment: FragmentedMessage) -> Option<Vec<u8>> {
        let is_final_fragment =
            fragment.header.fragment_index == fragment.header.fragment_count - 1;

        self.processing_fragments.push_back(fragment);

        if !is_final_fragment {
            return None;
        }

        Some(self.combine_fragments())
    }

    /// Combines all previously received fragments into a Vec<u8>.
    fn combine_fragments(&mut self) -> Vec<u8> {
        let mut combined_bytes = Vec::new();

        while let Some(fragment) = self.processing_fragments.pop_front() {
            combined_bytes.extend_from_slice(&fragment.bytes[0..fragment.header.fragment_size]);
        }

        combined_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_message() {
        let test_bytes = b"Hello world!";
        let fragmented_messages = MessageFragmenter::new(4).fragment_bytes(test_bytes.to_vec());

        assert_eq!(fragmented_messages[0].header.fragment_count, 3);
        assert_eq!(fragmented_messages[0].header.fragment_index, 0);
        assert_eq!(fragmented_messages[0].header.fragment_size, 4);
        assert_eq!(fragmented_messages[0].bytes, b"Hell");

        assert_eq!(fragmented_messages[2].header.fragment_count, 3);
        assert_eq!(fragmented_messages[2].header.fragment_index, 2);
        assert_eq!(fragmented_messages[2].header.fragment_size, 4);
        assert_eq!(fragmented_messages[2].bytes, b"rld!");
    }

    #[test]
    fn test_fragment_message_offset() {
        let test_bytes = b"Hello world!";
        let fragmented_messages = MessageFragmenter::new(5).fragment_bytes(test_bytes.to_vec());

        assert_eq!(fragmented_messages[0].header.fragment_count, 3);
        assert_eq!(fragmented_messages[0].header.fragment_index, 0);
        assert_eq!(fragmented_messages[0].header.fragment_size, 5);
        assert_eq!(fragmented_messages[0].bytes, b"Hello");

        assert_eq!(fragmented_messages[2].header.fragment_count, 3);
        assert_eq!(fragmented_messages[2].header.fragment_index, 2);
        assert_eq!(fragmented_messages[2].header.fragment_size, 2);
        assert_eq!(fragmented_messages[2].bytes, b"d!");
    }

    #[test]
    fn test_combine_fragments() {
        let test_bytes = b"Hello world!";

        let mut fragmenter = MessageFragmenter::new(4);

        let mut final_value = None;

        for fragment in fragmenter.fragment_bytes(test_bytes.to_vec()) {
            final_value = fragmenter.process_fragment(fragment)
        }

        assert!(final_value.is_some());
        assert_eq!(final_value.unwrap(), test_bytes)
    }
}
