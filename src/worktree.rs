/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use jwalk::WalkDir;
use std::path::Path;
use crate::DirEntry;

#[derive(Debug)]
pub struct WorkTreeError {
    message: String,
}
impl From<jwalk::Error> for WorkTreeError {
    fn from(err: jwalk::Error) -> WorkTreeError {
        WorkTreeError {
            message: err.to_string(),
        }
    }
}
/// A worktree of a repo.
///
/// Some common git internal terms.
///
/// - `oid` - Object ID.  This is often the SHA of an item.  It could be a commit, file blob, tree,
///     etc.
#[derive(Debug)]
pub struct WorkTree {
    path: String,
    pub entries: Vec<DirEntry>,
}

impl WorkTree {
    /// Returns the index for the git repo at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<WorkTree, WorkTreeError> {
        let mut entries = vec![];
        for entry in WalkDir::new(path).skip_hidden(false) {
            entries.push(DirEntry {sha: *b"00000000000000000000", name: entry?.path().to_str().ok_or(WorkTreeError{message: "FAIL WHALE".to_string()})?.to_string()});
            // println!("{}", entry?.path().display());
        }
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: vec![]
            // entries
        };
        Ok(work_tree)
    }
}
