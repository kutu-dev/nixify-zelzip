// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Management tool for multiple tasks on the monorepo.

use color_eyre::Result;
use tracing::info;
use util::setup_logging_for_cli;

mod cli;
mod root_path;
mod todo;

fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logging_for_cli();

    let root_path = root_path::get_root_path()?;
    let matches = cli::get_matches();

    if let Some(_matches) = matches.subcommand_matches("todo") {
        todo::print_tasks(&root_path)?;
    }

    if let Some(_matches) = matches.subcommand_matches("gen") {
        info!("Generating machine provided files...");
        cmd_lib::run_cmd! {
            cd $root_path;
            nix run .#generateFiles
        }?;
    }

    if let Some(_matches) = matches.subcommand_matches("check") {
        info!("Checking with the Nix build system");
        cmd_lib::run_cmd! {
            nix flake check --log-format internal-json $root_path |& nom --json
        }?;
    }

    if let Some(_matches) = matches.subcommand_matches("fix") {
        info!("Trying to fix as many files as possible");

        // `addlicense` can fail to due a bug in which it assumes
        // that any directory with a dot in its name is a file
        cmd_lib::run_cmd! {
            cd $root_path;

            alejandra .;
            taplo format --colors always .;

            cargo clippy --fix --allow-dirty;
            cargo fmt;

            cargo hakari generate;
            cargo hakari manage-deps --yes;

            addlicense -s -l mpl -ignore "**/*.*/**/*" .

            nix run .#generateFiles
        }?
    }

    Ok(())
}
