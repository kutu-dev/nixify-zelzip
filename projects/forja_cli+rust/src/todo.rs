// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use color_eyre::Result;
use std::path::Path;
use tracing::info;
use walkdir::WalkDir;

pub(crate) fn print_tasks(root_path: &Path) -> Result<()> {
    info!("Printing TODO.md files");

    for entry in WalkDir::new(root_path) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let file_name = match path.file_name() {
            Some(file_name) => file_name,
            None => continue,
        };

        if let Some("TODO.md") = file_name.to_str() {
            info!("Printing TODO file from {:?}", path);
            cmd_lib::run_cmd! {
                glow $path
            }?;
        }
    }

    Ok(())
}
