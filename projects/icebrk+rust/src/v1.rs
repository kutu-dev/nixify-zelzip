// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use derive_jserror::JsError;
use thiserror::Error;
use wasm_bindgen::prelude::*;

const HMAC_KEY_REGION_00: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_region_00.bin");
const HMAC_KEY_REGION_01: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_region_01.bin");
const HMAC_KEY_REGION_02: &[u8; 32] = include_bytes!("v1/3ds_hmac_key_region_02.bin");

#[derive(Error, JsError, Debug)]
#[allow(missing_docs)]
pub enum V1Error {
    #[error("The inquiry number has an unknown region encoded: {0}")]
    UnknownRegion(u8),
}

/// Calculate the master key for the parental control using the v1 algorithm. The inquire number
/// cannot be bigger than 10 digits and the date must be valid (there are some loose checks).
///
/// Remember that the given master key must be presented with the correct amount of leading zeroes
/// to always have 5 digits.
///
/// Only works on 3DS (from 7.0.0 to 7.1.0).
///
/// This function internal uses a set of HMAC keys, one for each region of the 3DS, at this moment
/// only the keys for the regions 0, 1 and 2 have been found.
#[wasm_bindgen]
pub fn calculate_v1_master_key(inquiry_number: u64, day: u8, month: u8) -> Result<u32, V1Error> {
    assert!(inquiry_number <= 9_999_999_999);

    assert!(day > 0);
    assert!(day <= 31);

    assert!(month > 0);
    assert!(month <= 12);

    let region = inquiry_number / 1_000_000_000;

    let hmac_key = match region {
        0x00 => HMAC_KEY_REGION_00,
        0x01 => HMAC_KEY_REGION_01,
        0x02 => HMAC_KEY_REGION_02,

        _ => return Err(V1Error::UnknownRegion(region as u8)),
    };

    Ok(crate::calculate_master_key_shared_v1_and_v2(
        hmac_key,
        inquiry_number,
        day,
        month,
        false,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u8 = 5;
    const MONTH: u8 = 8;

    #[test]
    fn region_0() {
        assert_eq!(
            calculate_v1_master_key(123456789, DAY, MONTH).unwrap(),
            3741
        );
    }

    #[test]
    fn region_1() {
        assert_eq!(
            calculate_v1_master_key(1123456789, DAY, MONTH).unwrap(),
            93328
        );
    }

    #[test]
    fn region_2() {
        assert_eq!(
            calculate_v1_master_key(2123456789, DAY, MONTH).unwrap(),
            10129
        );
    }
}
