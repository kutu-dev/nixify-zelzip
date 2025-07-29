// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

// WARNING: As written on the project `TODO.md` pretty much everything here
// should be moved to a extra lib & CLI (with a passthrough to Forja CLI)
// and properly documented for third-party usage.

mod comment_block;
mod inline_todo_entry;
mod resource;

use crate::todo::comment_block::CommentBlock;
use crate::todo::inline_todo_entry::{InlineTodoEntry, Tag};
use crate::todo::resource::Resource;
use color_eyre::eyre::OptionExt;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use colored::Colorize;
use core::num;
use serde::Deserialize;
use std::fmt::{self, format, write, Display, Formatter};
use std::io::Cursor;
use std::path::{self, PathBuf};
use std::{ffi::OsStr, fs, path::Path};
use tracing::{field, info};
use url::Url;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
enum GitHubApiIssueDataState {
    #[serde(rename = "open")]
    Open,

    #[serde(rename = "closed")]
    Closed,
}

#[derive(Deserialize, Debug)]
struct GitHubApiIssueData {
    state: GitHubApiIssueDataState,
}

pub(crate) fn print_tasks(root_path: &Path) -> Result<()> {
    let search_result = SearchResult::search_in_path(root_path)?;

    info!("Printing TODO.md files");
    for path in search_result.todo_paths {
        info!("Printing TODO file from {:?}", path);
        cmd_lib::run_cmd! {
            glow $path
        }?;
    }

    let mut blocks: Vec<CommentBlock> = vec![];

    for path in search_result.pound_sign_prefixed_comments_file_paths {
        add_comment_blocks_from_path(&mut blocks, &path, "#")?;
    }

    for path in search_result.double_slash_prefixed_comments_file_paths {
        add_comment_blocks_from_path(&mut blocks, &path, "//")?;
    }

    let entries: Vec<InlineTodoEntry> = blocks.iter().map(InlineTodoEntry::new).flatten().collect();

    // NOTE: The output doesn't have to be valid YAML
    info!("Inline TODO entries:");
    for entry in entries {
        info!("  - Title: {}", entry.title.bold().underline());
        info!("    Content:");

        for line in entry.content {
            info!("      {line}");
        }

        if entry.tags.len() >= 1 {
            info!("    Tags:");
        }

        for tag in entry.tags {
            let tag = match tag {
                Tag::Fix => "Fix".purple().bold(),
                Tag::Improve => "Improve".blue().bold(),
                Tag::Roadblock => "Roadblock".red().bold(),
                Tag::Unknown(ref tag) => format!("Unknown: {tag}").as_str().bright_black(),
            };

            info!("      - {tag}");
        }

        if entry.resources.len() >= 1 {
            info!("    Resources:");
        }

        for resource in entry.resources {
            match resource {
                Resource::GitHubIssue {
                    owner,
                    repository,
                    id,
                } => {
                    let url = format!("https://github.com/{owner}/{repository}/issues/{id}");

                    let client = reqwest::blocking::Client::new();

                    let issue_data = client
                        .get(format!(
                            "https://api.github.com/repos/{owner}/{repository}/issues/{id}"
                        ))
                        .header("Accept", "application/vnd.github+json")
                        .header("X-GitHub-Api-Version", "2022-11-28")
                        .header(
                            "User-Agent",
                            "Forja CLI for the ZELZIP project, contact: GitHub user @kutu-dev",
                        )
                        .send()?
                        .json::<GitHubApiIssueData>()?;

                    let state = match issue_data.state {
                        GitHubApiIssueDataState::Open => "Open".bright_green().bold(),
                        GitHubApiIssueDataState::Closed => "Closed".bright_purple().bold(),
                    };

                    info!("      - GitHub Issue:");
                    info!("        - State: {state}");
                    info!("        - Url: {}", url.bright_cyan().underline());
                }

                Resource::Url(url) => {
                    info!("      - Url: {}", url.bright_cyan().underline());
                }
            }
        }

        let root_path_prefix = root_path
            .to_str()
            .ok_or_eyre("The path of the given file is not a valid UTF-8 string")?;

        let mut file_path = entry
            .path
            .to_string_lossy()
            .trim_start_matches(root_path_prefix)
            .to_owned();

        file_path.insert(0, '/');

        info!("    File: {}", file_path.bright_black());
        info!("    Line: {}", entry.line_number.bright_black());

        info!("")
    }

    Ok(())
}

#[derive(Debug)]
struct SearchResult {
    todo_paths: Vec<PathBuf>,
    double_slash_prefixed_comments_file_paths: Vec<PathBuf>,
    pound_sign_prefixed_comments_file_paths: Vec<PathBuf>,
}

impl SearchResult {
    fn search_in_path(root_path: &Path) -> Result<Self> {
        let mut search_result = SearchResult {
            todo_paths: vec![],
            double_slash_prefixed_comments_file_paths: vec![],
            pound_sign_prefixed_comments_file_paths: vec![],
        };

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
                search_result.todo_paths.push(path.to_owned());
                continue;
            };

            let extension = match path.extension().and_then(|ext| ext.to_str()) {
                Some(extension) => extension,
                None => continue,
            };

            if let "rs" | "json5" = extension {
                search_result
                    .double_slash_prefixed_comments_file_paths
                    .push(path.to_owned());
                continue;
            }

            if let "nix" | "toml" | "yaml" = extension {
                search_result
                    .pound_sign_prefixed_comments_file_paths
                    .push(path.to_owned());
                continue;
            }
        }

        Ok(search_result)
    }
}

fn add_comment_blocks_from_path(
    blocks: &mut Vec<CommentBlock>,
    path: &Path,
    comment_start_pattern: &'static str,
) -> Result<()> {
    let text = fs::read_to_string(&path)?;
    let text = text.split('\n');

    let mut current_block: Option<CommentBlock> = None;

    for (line_number, line) in text.enumerate() {
        let line = line.trim_start();

        if !line.starts_with(comment_start_pattern) {
            if let Some(current_block) = current_block {
                blocks.push(current_block);
            }

            current_block = None;
            continue;
        }

        let line = line.trim_start_matches(comment_start_pattern).to_string();

        match current_block {
            Some(ref mut current_block) => current_block.comment.push(line),

            None => {
                current_block = Some(CommentBlock {
                    path: path.to_owned(),
                    line_number,
                    comment: vec![line],
                });
            }
        }
    }

    Ok(())
}
