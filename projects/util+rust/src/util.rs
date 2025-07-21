// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Miscellaneous code shared for all the crates inside the ZEL.ZIP project.
//!
//! Has partial support for `no_std` mode by disabling the default `std` feature flag. Extra suport
//! for "alloc-compatible" `no_std` environments is available by enabling the `alloc` feature flag.

mod extensions;
mod macros;

#[allow(unused_imports)]
pub use extensions::*;

mod aes;
pub mod logging;
mod recall_view;
mod stream_pin;
mod view;

pub use aes::{Aes128CbcDec, AesCbcStream};
pub use logging::setup_logging_for_cli;
pub use recall_view::RecallView;
pub use stream_pin::StreamPin;
pub use view::View;

/// Align a value to the next multiple of the given boundary.
pub fn align_to_boundary(value: u64, boundary: u64) -> u64 {
    if value == 0 {
        return 0;
    }

    value + (boundary - (value % boundary)) % boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_to_boundary_unaligned_value() {
        assert_eq!(align_to_boundary(117, 64), 128);
    }

    #[test]
    fn align_to_boundary_aligned_value() {
        assert_eq!(align_to_boundary(100, 50), 100);
    }

    #[test]
    fn align_to_boundary_same_value() {
        assert_eq!(align_to_boundary(73, 73), 73);
    }

    #[test]
    fn align_to_boundary_zero() {
        assert_eq!(align_to_boundary(0, 0), 0);
    }
}
