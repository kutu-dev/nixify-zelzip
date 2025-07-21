// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use clap::{command, ArgMatches, Command};

pub(crate) fn get_matches() -> ArgMatches {
    command!()
        .subcommand_required(true)
        .subcommand(Command::new("todo").about("Print the current tasks to do"))
        .subcommand(Command::new("check").about("Check the quality of the code"))
        .subcommand(Command::new("fix").about("Try to fix issues in the code"))
        .get_matches()
}
