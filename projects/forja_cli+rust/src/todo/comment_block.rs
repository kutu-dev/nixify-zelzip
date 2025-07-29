use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct CommentBlock {
    pub(crate) path: PathBuf,
    pub(crate) line_number: usize,

    pub(crate) comment: Vec<String>,
}
