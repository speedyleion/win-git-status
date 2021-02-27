/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::cmp::Ordering;
use jwalk::{WalkDir, WalkDirGeneric};
use std::path::Path;
use crate::DirEntry;
use crate::Index;

#[derive(Debug)]
enum Status {
    CURRENT,
    NEW,
    MODIFIED,
    DELETED
}

impl Default for Status {
    fn default() -> Self { Status::MODIFIED }
}

#[derive(Debug, Default)]
pub struct StatusState {
    state: Status,
}

#[derive(Debug, Default, Clone)]
pub struct WorkTreeState {
    index: Index,
}

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
            entries.push(DirEntry {mtime: 0, size: 0, sha: *b"00000000000000000000", name: entry?.path().to_str().ok_or(WorkTreeError{message: "FAIL WHALE".to_string()})?.to_string()});
            // println!("{}", entry?.path().display());
        }
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: vec![]
            // entries
        };
        Ok(work_tree)
    }

    /// Compares an index to the on disk work tree.
    ///
    /// # Argumenst
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    /// * `index` - The index to compare against
    pub fn diff_against_index(path: &Path, index: &Index) -> Result<WorkTree, WorkTreeError> {
        let walk_dir = WalkDirGeneric::<((WorkTreeState),(StatusState))>::new(path)
            .process_read_dir(|depth, path, read_dir_state, children| {
                // Sort to make it easier to compare the index and working tree for new versus deleted files.
                children.sort_by(|a, b| match (a, b) {
                    (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => Ordering::Equal,
                });

                children.first_mut().map(|dir_entry_result| {
                    if let Ok(dir_entry) = dir_entry_result {
                        dir_entry.client_state.state = Status::MODIFIED;
                    }
                });
                print!("{:?}", children);
            });
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: vec![]
            // entries
        };
        Ok(work_tree)

    }
}
