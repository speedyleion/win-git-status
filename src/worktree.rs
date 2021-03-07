/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use jwalk::WalkDirGeneric;
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::Index;
use std::time::SystemTime;

#[derive(PartialEq, Eq, Debug)]
pub enum Status {
    CURRENT,
    // NEW,
    MODIFIED,
    // DELETED
}

impl Default for Status {
    fn default() -> Self {
        Status::CURRENT
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct WorkTreeEntry {
    pub name: String,
    pub state: Status,
}

#[derive(Debug, Default, Clone)]
struct IndexState {
    path: PathBuf,
    index: Arc<Index>,
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
    pub entries: Vec<WorkTreeEntry>,
}

impl WorkTree {
    /// Compares an index to the on disk work tree.
    ///
    /// # Argumenst
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    /// * `index` - The index to compare against
    pub fn diff_against_index(path: &Path, index: Index) -> Result<WorkTree, WorkTreeError> {
        let index_state = IndexState {
            path: PathBuf::from(path),
            index: Arc::new(index),
        };
        let walk_dir = WalkDirGeneric::<(IndexState, WorkTreeEntry)>::new(path)
            .skip_hidden(false)
            .sort(true)
            .root_read_dir_state(index_state)
            .process_read_dir(process_directory);
        let mut entries = vec![];
        for entry in walk_dir {
            let entry = entry.unwrap();

            // Leverage the fact that `read_children_path` is set to None for files
            match entry.read_children_path {
                None => {
                    if let Status::MODIFIED = entry.client_state.state {
                        entries.push(entry.client_state)
                    }
                }
                _ => continue,
            }
        }
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries,
        };
        Ok(work_tree)
    }
}

fn process_directory(
    depth: Option<usize>,
    path: &Path,
    read_dir_state: &mut IndexState,
    children: &mut Vec<Result<jwalk::DirEntry<(IndexState, WorkTreeEntry)>, jwalk::Error>>,
) {
    // jwalk will use None for depth on the parent of the root path, not sure why...
    let _depth = match depth {
        Some(depth) => depth,
        None => return,
    };

    // Skip '.git' directory
    children.retain(|dir_entry_result| {
        dir_entry_result
            .as_ref()
            .map(|dir_entry| {
                dir_entry
                    .file_name
                    .to_str()
                    .map(|s| s != ".git")
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    });
    let index = &read_dir_state.index;
    let relative_path = diff_paths(path, &read_dir_state.path).unwrap();
    let index_dir_entry = index.entries.get(relative_path.to_str().unwrap()).unwrap();
    for child in children {
        if let Ok(child) = child {
            let meta = child.metadata().unwrap();
            let mtime = meta
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            let size = meta.len() as u32;
            let entry = &index_dir_entry[0];
            child.client_state.name = child.file_name.to_str().unwrap().to_string();
            if entry.mtime != mtime || entry.size != size {
                child.client_state.state = Status::MODIFIED;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DirEntry;
    use std::fs;
    use std::time::SystemTime;
    use temp_testdir::TempDir;

    // Test helper function to build up a temporary directory of `files`.  All files will have the
    // same contents `what\r\nis\r\nit`.  The `Index` will be populated with the values as the files
    // currently are on disk.  Callers can modify the returned `Index` to create differences or
    // create and delete files from the returned `TempDir`.
    fn temp_tree(files: Vec<&Path>) -> (Index, TempDir) {
        let temp_dir = TempDir::default();
        let mut index = Index::default();

        let file_contents = "what\r\nis\r\nit";
        for file in files {
            let full_path = temp_dir.join(file);

            // Done this way to support nested files
            fs::create_dir_all(full_path.parent().unwrap()).unwrap();
            fs::write(&full_path, file_contents).unwrap();
            let metadata = fs::metadata(&full_path).unwrap();

            let relative_parent = file.parent().unwrap().to_str().unwrap().to_string();
            let dir_entries = index.entries.entry(relative_parent).or_insert(vec![]);
            dir_entries.push(DirEntry {
                mtime: metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
                size: metadata.len() as u32,
                sha: [0; 20],
                name: file.to_str().unwrap().to_string(),
            });
        }
        (index, temp_dir)
    }

    #[test]
    fn test_diff_against_index_nothing_modified() {
        let (index, temp_dir) = temp_tree(vec![Path::new("simple_file.txt")]);
        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        assert_eq!(value.entries, vec![]);
    }

    #[test]
    fn test_diff_against_index_a_file_modified() {
        let entry_name = "simple_file.txt";
        let (mut index, temp_dir) = temp_tree(vec![Path::new(entry_name)]);
        let dir_entries = index.entries.get_mut("").unwrap();
        dir_entries[0].size += 1;
        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        let entries = vec![WorkTreeEntry {
            name: entry_name.to_string(),
            state: Status::MODIFIED,
        }];
        assert_eq!(value.entries, entries);
    }
}
