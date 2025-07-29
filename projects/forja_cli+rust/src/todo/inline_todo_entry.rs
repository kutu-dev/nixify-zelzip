// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::todo::resource::Resource;
use crate::todo::CommentBlock;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct InlineTodoEntry {
    pub(crate) path: PathBuf,
    pub(crate) line_number: usize,

    pub(crate) title: String,
    pub(crate) content: Vec<String>,

    pub(crate) resources: Vec<Resource>,
    pub(crate) tags: Vec<Tag>,
}

impl InlineTodoEntry {
    pub(crate) fn new(block: &CommentBlock) -> Option<Self> {
        let first_line = match block.comment.first() {
            Some(first_line) => first_line.trim(),
            None => {
                return None;
            }
        };

        let (title, tags) = if first_line.starts_with("TODO: ") {
            let title = first_line.trim_start_matches("TODO: ");

            (title, vec![])
        } else if first_line.starts_with("TODO(") {
            let components: Vec<&str> = first_line
                .trim_start_matches("TODO(")
                .split("): ")
                .collect();

            if components.len() < 2 {
                return None;
            }

            let tags = components[0]
                .split(",")
                .map(|label| label.trim())
                .map(Tag::new)
                .collect();

            let title = components[1];

            (title, tags)
        } else {
            return None;
        };

        let content = block.comment[1..].to_owned();
        let resources = content
            .iter()
            .flat_map(|line| line.split_whitespace())
            .filter_map(Resource::try_find_resource)
            .collect();

        Some(Self {
            path: block.path.clone(),
            line_number: block.line_number,
            title: String::from(title),
            content,
            resources,
            tags,
        })
    }
}

#[derive(Debug)]
pub(crate) enum Tag {
    Improve,
    Roadblock,
    Fix,
    Unknown(String),
}

impl Tag {
    fn new(label: &str) -> Self {
        match label {
            "IMPROVE" => Self::Improve,
            "ROADBLOCK" => Self::Roadblock,
            "FIX" => Self::Fix,

            _ => Self::Unknown(String::from(label)),
        }
    }
}
