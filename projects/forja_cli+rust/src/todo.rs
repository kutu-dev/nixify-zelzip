// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use color_eyre::Result;
use core::num;
use std::fmt::{self, format, write, Display, Formatter};
use std::{ffi::OsStr, fs, path::Path};
use tracing::info;
use walkdir::WalkDir;

#[derive(Debug)]
enum Tag {
    Improve,
    Fix,
    Track(TrackTag),
    Discover,
    Unknown(String),
}

#[derive(Debug)]
enum TrackTag {
    GitHubIssue(u32),
    Unknown(String),
}

impl Tag {
    fn new(raw_tag: &str) -> Self {
        let raw_tag = &raw_tag[1..raw_tag.len() - 1];

        if raw_tag.contains("TRACK") {
            if let Some(track) = Self::parse_track_tag(raw_tag) {
                return Self::Track(track);
            }
        }

        match raw_tag {
            "IMPROVE" => Self::Improve,
            "FIX" => Self::Fix,
            "DISCOVER" => Self::Discover,

            _ => Self::Unknown(raw_tag.to_string()),
        }
    }

    fn parse_track_tag(raw_tag: &str) -> Option<TrackTag> {
        let url = raw_tag.trim_start_matches("TRACK: ");
        Some(Self::Track(url.to_string()))
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Improve => String::from("IMPROVE"),
            Self::Fix => String::from("FIX"),
            Self::Track(url) => format!("TRACK -> {url}"),
            Self::Discover => String::from("DISCOVER"),
            Self::Unknown(unknown_tag) => format!("{unknown_tag} (unknown)"),
        };

        write!(f, "{text}")?;

        Ok(())
    }
}

fn reduce_whitespaces(lines: &mut [String]) {
    let mut smallest_whitespace_count = usize::MAX;

    for line in &mut *lines {
        let mut whitespace_count = 0;

        for char in line.chars() {
            if char != ' ' {
                break;
            }

            whitespace_count += 1;
        }

        smallest_whitespace_count = smallest_whitespace_count.min(whitespace_count);
    }

    for line in lines {
        *line = line[smallest_whitespace_count..].to_string();
    }
}

#[derive(Debug)]
struct InlineTodoEntry {
    message: Vec<String>,
    line: usize,
    tag: Option<Tag>,
}

impl InlineTodoEntry {
    fn new(numbered_line: (usize, &str), lines: &[&str]) -> Option<Self> {
        let (line, text) = numbered_line;

        // "Tagless" entry
        if text.starts_with(": ") {
            return Some(InlineTodoEntry {
                message: Self::parse_message(text, line, lines),
                line,
                tag: None,
            });
        }

        // Entry with a tag
        if text.starts_with("(") {
            let mut escape = false;
            let mut is_tag = true;

            let mut tag_chars = vec![];
            let mut message_chars = vec![];

            for char in text.chars() {
                if escape {
                    continue;
                }

                if char == '\\' {
                    escape = true;
                    continue;
                }

                if is_tag {
                    tag_chars.push(char);
                } else {
                    message_chars.push(char);
                }

                if char == ')' {
                    is_tag = false;
                }
            }

            let raw_tag: String = tag_chars.into_iter().collect();
            let message: String = message_chars.into_iter().collect();

            return Some(InlineTodoEntry {
                message: Self::parse_message(&message, line, lines),
                line,
                tag: Some(Tag::new(&raw_tag)),
            });
        }

        None
    }

    fn parse_message(text: &str, line: usize, lines: &[&str]) -> Vec<String> {
        let text = text.trim_start_matches(": ");

        if text == "|>|" {
            return lines[line + 1..]
                .iter()
                .map(|line| line.trim())
                .take_while(|line| line.starts_with("# "))
                .map(|line| line.trim_start_matches("# "))
                .map(|line| line.to_owned())
                .collect();
        };

        vec![text.to_owned()]
    }
}

pub(crate) fn print_tasks(root_path: &Path) -> Result<()> {
    let mut double_slash_files = vec![];
    let mut pound_sign_files = vec![];
    info!("Printing TODO.md files");

    for entry in WalkDir::new(root_path) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(file_name) => file_name,
            None => continue,
        };

        if let "TODO.md" = file_name {
            info!("Printing TODO file from {:?}", path);
            cmd_lib::run_cmd! {
                glow $path
            }?;

            continue;
        };

        let extension = match path.extension().and_then(|ext| ext.to_str()) {
            Some(extension) => extension,
            None => continue,
        };

        if let "rs" | "json5" = extension {
            double_slash_files.push(path.to_owned());
            continue;
        }

        if let "nix" | "toml" | "yaml" = extension {
            pound_sign_files.push(path.to_owned());
            continue;
        }
    }

    info!("Parsing inline TODO entries:");
    for path in pound_sign_files {
        let text = fs::read_to_string(&path)?;

        let lines: Vec<&str> = text.split('\n').collect();

        let entries_in_file = lines
            .iter()
            .map(|line| line.trim())
            .enumerate()
            .filter(|(_, line)| line.starts_with("# TODO"))
            .map(|(i, line)| (i, line.trim_start_matches("# TODO")))
            .map(|numbered_line| InlineTodoEntry::new(numbered_line, &lines))
            .flatten();

        for mut entry in entries_in_file {
            let root_path = &root_path.to_string_lossy().into_owned();

            let path = path.to_string_lossy();
            let path = path.trim_start_matches(root_path);

            info!("  - Entry:");
            info!("    - File: /{}", path);
            info!("    - Line: {}", entry.line);

            if let Some(tag) = entry.tag {
                info!("    - Tag: {}", tag)
            }

            reduce_whitespaces(&mut entry.message);

            info!("    - Message: {}", entry.message[0]);
            for line in &entry.message[1..] {
                info!("               {line}");
            }

            info!("")
        }
    }

    Ok(())
}
