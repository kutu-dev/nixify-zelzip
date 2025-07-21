// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use std::env;
use std::path::{Path, PathBuf};

const BEACON_FILENAME: &str = "forja-root-beacon.txt";

/// Try to find the root of the monorepo with recursion,
/// the given `path` must always point to a valid directory.
fn check_directory(directory_path: &Path) -> Result<PathBuf> {
    let mut beacon_path = directory_path.to_owned();
    beacon_path.push(BEACON_FILENAME);

    if beacon_path.is_file() {
        return Ok(directory_path.to_owned());
    }

    check_directory(
        directory_path
            .parent()
            .wrap_err_with(|| "Failed to find the root of the monorepo".to_string())?,
    )
}

pub(crate) fn get_root_path() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    check_directory(&current_dir)
}
