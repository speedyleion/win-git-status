/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use jwalk::WalkDirGeneric;
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use core::cmp::Ordering;

use crate::{Index, DirEntry};
use std::time::SystemTime;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    CURRENT,
    NEW,
    MODIFIED,
    DELETED
}

impl Default for Status {
    fn default() -> Self {
        Status::CURRENT
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct WorkTreeEntry {
    pub name: String,
    pub state: Status,
}

#[derive(Debug, Default, Clone)]
struct IndexState {
    path: PathBuf,
    index: Arc<Index>,
    changed_files: Arc<Mutex<Vec<WorkTreeEntry>>>,
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
        let changed_files = Arc::new(Mutex::new(vec![]));
        let entries = Arc::clone(&changed_files);

        let index_state = IndexState {
            path: PathBuf::from(path),
            index: Arc::new(index),
            changed_files,
        };
        let walk_dir = WalkDirGeneric::<(IndexState, WorkTreeEntry)>::new(path)
            .skip_hidden(false)
            .sort(true)
            .root_read_dir_state(index_state)
            .process_read_dir(process_directory);

        for _ in walk_dir {
            continue;
        }

        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: entries.lock().unwrap().to_vec(),
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
    children.retain(|dir_entry_result| {
        dir_entry_result
            .as_ref()
            .map(|dir_entry| dir_entry.file_type().is_file())
            .unwrap_or(false)
    });
    let index = &read_dir_state.index;
    let relative_path = diff_paths(path, &read_dir_state.path).unwrap();
    let unix_path = relative_path.to_str().unwrap().replace("\\", "/");

    let index_dir_entry = index.entries.get(&unix_path).unwrap();

    get_file_deltas(children, index_dir_entry, &read_dir_state.changed_files);
}

fn get_file_deltas(
    worktree: &mut Vec<Result<jwalk::DirEntry<(IndexState, WorkTreeEntry)>, jwalk::Error>>,
    index: &Vec<DirEntry>,
    file_changes: &Mutex<Vec<WorkTreeEntry>>,
    ) {
    let mut worktree_iter = worktree.iter_mut();
    let mut index_iter = index.iter();
    let mut worktree_file = worktree_iter.next();
    let mut index_file = index_iter.next();
    loop {
        if let Some(wa_file) = worktree_file.as_mut() {
            let mut w_file = wa_file.as_mut().unwrap();
            match index_file {
                Some(i_file) => {
                    match w_file.file_name().cmp(i_file.name.as_ref()) {
                        Ordering::Equal => {
                            if is_modified(&mut w_file, i_file) {
                                file_changes.lock().unwrap().push(WorkTreeEntry{name: w_file.file_name.to_str().unwrap().to_string(), state: Status::MODIFIED,});
                            }
                            index_file = index_iter.next();
                            worktree_file = worktree_iter.next();
                        },
                        Ordering::Less => {
                            file_changes.lock().unwrap().push(WorkTreeEntry{name: w_file.file_name.to_str().unwrap().to_string(), state: Status::NEW,});
                            worktree_file = worktree_iter.next();
                        }
                        Ordering::Greater => {
                            file_changes.lock().unwrap().push(WorkTreeEntry{name: i_file.name.to_string(), state: Status::DELETED,});
                            index_file = index_iter.next();
                        }
                    }
                },
                None => {
                    file_changes.lock().unwrap().push(WorkTreeEntry{name: w_file.file_name.to_str().unwrap().to_string(), state: Status::NEW,});
                    worktree_file = worktree_iter.next();
                },
            }

        } else {
            // handle index has new entries...
            break;
        }
    }
}

fn is_modified(worktree_file: &mut jwalk::DirEntry<(IndexState, WorkTreeEntry)>, index_file: &DirEntry) -> bool {
    let meta = worktree_file.metadata().unwrap();
    let mtime = meta.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32;
    let size = meta.len() as u32;
    let mut modified = false;
    if index_file.mtime != mtime || index_file.size != size {
        modified = true;
    }
    modified
}

#[cfg(test)]
mod tests {
    use super::*;
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
            for ancestor in Path::new(&relative_parent).ancestors() {
                index
                    .entries
                    .entry(ancestor.to_str().unwrap().to_string())
                    .or_insert_with(Vec::<DirEntry>::new);
            }

            let dir_entries = index.entries.get_mut(&relative_parent).unwrap();
            dir_entries.push(DirEntry {
                mtime: metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32,
                size: metadata.len() as u32,
                sha: [0; 20],
                name: file.file_name().unwrap().to_str().unwrap().to_string(),
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

    #[test]
    fn test_diff_against_index_deeply_nested() {
        let (index, temp_dir) = temp_tree(vec![Path::new("dir_1/dir_2/dir_3/file.txt")]);
        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        assert_eq!(value.entries, vec![]);
    }

    #[test]
    fn test_new_file_in_worktree() {
        let (index, temp_dir) = temp_tree(vec![Path::new("simple_file.txt")]);
        let new_file_name = "new_file.txt";
        let new_file = temp_dir.join(new_file_name);
        fs::create_dir_all(new_file.parent().unwrap()).unwrap();
        fs::write(&new_file, "stuff").unwrap();

        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        let entries = vec![WorkTreeEntry {
            name: new_file_name.to_string(),
            state: Status::NEW,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_multiple_new_files_in_worktree() {
        let (index, temp_dir) = temp_tree(vec![Path::new("simple_file.txt")]);

        // Putting them in order for the simpler assert
        let new_file_names = vec!["a_file.txt", "z_file.txt"];
        for name in &new_file_names {
            let new_file = temp_dir.join(&name);
            fs::create_dir_all(new_file.parent().unwrap()).unwrap();
            fs::write(&new_file, "stuff").unwrap();
        }

        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        let entries: Vec<WorkTreeEntry> = new_file_names.iter().map(|&n| WorkTreeEntry {
            name: n.to_string(),
            state: Status::NEW,
        }).collect();
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_deleted_file_in_worktree() {
        let names = vec!["file_1.txt", "file_2.txt", "foo.txt"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let (index, temp_dir) = temp_tree(files);
        fs::remove_file(temp_dir.join("file_2.txt")).unwrap();

        let value = WorkTree::diff_against_index(&*temp_dir, index).unwrap();
        let entries = vec![WorkTreeEntry {
            name: "file_2.txt".to_string(),
            state: Status::DELETED,
        }];
        assert_eq!(value.entries, entries);
    }

}
