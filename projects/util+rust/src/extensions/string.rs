// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::string::{FromUtf8Error, String};

/// Extension trait of [String] with useful miscellaneous operations.
pub trait StringEx {
    /// Create a [String] from a null-terminated string buffer.
    fn from_null_terminated_bytes(buffer: &[u8]) -> Result<String, FromUtf8Error> {
        let string_end = buffer
            .iter()
            .position(|&char| char == b'\0')
            // Fallback to use all the bytes
            .unwrap_or(buffer.len());

        String::from_utf8(buffer[0..string_end].to_vec())
    }
}

impl StringEx for String {}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_TEXT_ASCII: [u8; 3] = [72, 105, 33];
    const DUMMY_TEXT_STR: &str = "Hi!";

    #[test]
    fn string_from_null_terminated_bytes_no_null_char() {
        assert_eq!(
            String::from_null_terminated_bytes(&DUMMY_TEXT_ASCII).unwrap(),
            DUMMY_TEXT_STR
        );
    }

    #[test]
    fn string_from_null_terminated_bytes_with_null_chars() {
        let mut buffer = Vec::from(DUMMY_TEXT_ASCII);
        buffer.append(&mut vec![0, 0, 0]);

        assert_eq!(
            String::from_null_terminated_bytes(&DUMMY_TEXT_ASCII).unwrap(),
            DUMMY_TEXT_STR
        );
    }
}
