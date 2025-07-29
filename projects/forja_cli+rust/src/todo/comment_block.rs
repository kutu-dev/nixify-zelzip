// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct CommentBlock {
    pub(crate) path: PathBuf,
    pub(crate) line_number: usize,

    pub(crate) comment: Vec<String>,
}
