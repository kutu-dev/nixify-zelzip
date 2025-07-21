// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Logging management.

use tracing_subscriber::fmt;

/// Setup logging with the `tracing` crate, tailored for CLI applications.
pub fn setup_logging_for_cli() {
    let format = fmt::format().without_time();
    tracing_subscriber::fmt().event_format(format).init();
}
