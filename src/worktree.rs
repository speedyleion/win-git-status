/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::cmp::Ordering;
use jwalk::{WalkDir, WalkDirGeneric};
use std::path::Path;
use std::sync::{Mutex, Arc};

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
pub struct IndexState {
    index: Arc<Mutex<Index>>,
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
    /// Compares an index to the on disk work tree.
    ///
    /// # Argumenst
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    /// * `index` - The index to compare against
    pub fn diff_against_index(path: &Path, index: &Index) -> Result<WorkTree, WorkTreeError> {
        let walk_dir = WalkDirGeneric::<((IndexState),(StatusState))>::new(path).skip_hidden(false).sort(true)
            .process_read_dir(process_directory);
        let mut entries = vec![];
        for entry in walk_dir {
            entries.push(DirEntry {mtime: 0, size: 0, name: entry?.path().to_str().ok_or(WorkTreeError{message: "FAIL WHALE".to_string()})?.to_string(), ..Default::default()});
        }
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: entries
            // entries
        };
        Ok(work_tree)

    }
}

fn process_directory(depth: Option<usize>, path: &Path, read_dir_state: &mut IndexState, children: &mut Vec<Result<jwalk::DirEntry<((IndexState),(StatusState))>, jwalk::Error>>){
    // jwalk will use None for depth on the parent of the root path, not sure why...
    let depth = match depth {
        Some(depth) => depth,
        None => return,
    };

    // Skip '.git' directory
    children.retain(|dir_entry_result| {
        dir_entry_result.as_ref().map(|dir_entry| {
            dir_entry.file_name
                .to_str()
                .map(|s| s != ".git")
                .unwrap_or(false)
        }).unwrap_or(false)
    });
    children.first_mut().map(|dir_entry_result| {
        if let Ok(dir_entry) = dir_entry_result {
            dir_entry.client_state.state = Status::MODIFIED;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_testdir::TempDir;
    use std::fs;

    #[test]
    fn test_diff_against_index() {
        let temp_dir = TempDir::default();
        let file_contents = "what\r\nis\r\nit";
        let entry_name = "simple_file.txt";
        let file = temp_dir.join(entry_name);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, file_contents).unwrap();
        let index =  Index::default();
        let value = WorkTree::diff_against_index(&*temp_dir, &index).unwrap();
        assert_eq!(value.entries.len(), 1);

    }
}

