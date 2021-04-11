/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use core::cmp::Ordering;
use jwalk::{Parallelism, WalkDirGeneric};
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::direntry::{DirEntry, ObjectType};
use crate::dirstat::DirectoryStat;
use crate::error::StatusError;
use crate::status::{Status, StatusEntry};
use crate::Index;
use git2::Repository;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs;

#[derive(Debug, Default, Clone)]
struct IndexState {
    path: PathBuf,
    index: Arc<Index>,
    changed_files: Arc<Mutex<Vec<StatusEntry>>>,
    ignores: Vec<Arc<Gitignore>>,
}

impl From<jwalk::Error> for StatusError {
    fn from(err: jwalk::Error) -> StatusError {
        StatusError {
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
    pub entries: Vec<StatusEntry>,
}

impl WorkTree {
    /// Compares an index to the on disk work tree.
    ///
    /// # Arguments
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    /// * `index` - The index to compare against
    pub fn diff_against_index(path: &Path, index: Index) -> Result<WorkTree, StatusError> {
        WorkTree::diff_against_index_recursive(path, index, true)
    }

    pub fn diff_against_index_recursive(
        path: &Path,
        index: Index,
        root: bool,
    ) -> Result<WorkTree, StatusError> {
        let changed_files = Arc::new(Mutex::new(vec![]));
        let entries = Arc::clone(&changed_files);

        let (global_ignore, _) = GitignoreBuilder::new("").build_global();
        let index_state = IndexState {
            path: PathBuf::from(path),
            index: Arc::new(index),
            changed_files,
            ignores: vec![Arc::new(global_ignore)],
        };

        let parallelism = match root {
            true => Parallelism::RayonDefaultPool,
            false => Parallelism::Serial,
        };

        let walk_dir = WalkDirGeneric::<(IndexState, bool)>::new(path)
            .skip_hidden(false)
            .sort(true)
            .root_read_dir_state(index_state)
            .process_read_dir(process_directory)
            .parallelism(parallelism);

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
    children: &mut Vec<Result<jwalk::DirEntry<(IndexState, bool)>, jwalk::Error>>,
) {
    // jwalk will use None for depth on the parent of the root path, not sure why...
    let _depth = match depth {
        Some(depth) => depth,
        None => return,
    };

    update_ignores(path, &mut read_dir_state.ignores);

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
    let unix_path = relative_path.to_str().unwrap().replace("\\", "/");

    let index_dir_entry = index.entries.get(&unix_path);

    match index_dir_entry {
        // None happens when dealing with an empty repo, normally we don't have empty index
        // directories, since git tracks files not directories
        None => {}
        Some(dir_entry) => get_file_deltas(children, dir_entry, index, read_dir_state),
    }
}

fn update_ignores(path: &Path, ignores: &mut Vec<Arc<Gitignore>>) {
    let ignore_file = path.join(".gitignore");
    if !ignore_file.exists() {
        return;
    }
    let mut builder = GitignoreBuilder::new(path);
    builder.add(ignore_file);
    let ignore = builder.build().unwrap();
    ignores.insert(0, Arc::new(ignore));
}

fn get_file_deltas(
    worktree: &mut Vec<Result<jwalk::DirEntry<(IndexState, bool)>, jwalk::Error>>,
    index_entry: &[DirEntry],
    index: &Arc<Index>,
    read_dir_state: &IndexState,
) {
    let file_changes = &read_dir_state.changed_files;
    let mut worktree_iter = worktree.iter_mut();
    let mut index_iter = index_entry.iter();
    let mut worktree_file = worktree_iter.next();
    let mut index_file = index_iter.next();
    let mut stats = None;
    while let Some(wa_file) = worktree_file.as_mut() {
        let w_file = wa_file.as_mut().unwrap();
        match index_file {
            Some(i_file) => match w_file.file_name().cmp(i_file.name.as_ref()) {
                Ordering::Equal => {
                    if let Some(entry) = process_tracked_item(w_file, i_file, &mut stats, file_changes) {
                        file_changes.lock().unwrap().push(entry);
                    }
                    index_file = index_iter.next();
                    worktree_file = worktree_iter.next();
                }
                Ordering::Less => {
                    if let Some(entry) = process_new_item(w_file, index, &read_dir_state.ignores) {
                        file_changes.lock().unwrap().push(entry);
                    }
                    worktree_file = worktree_iter.next();
                }
                Ordering::Greater => {
                    if let Some(entry) = process_deleted_item(i_file) {
                        file_changes.lock().unwrap().push(entry);
                    }
                    index_file = index_iter.next();
                }
            },
            None => {
                if let Some(entry) = process_new_item(w_file, index, &read_dir_state.ignores) {
                    file_changes.lock().unwrap().push(entry);
                }
                worktree_file = worktree_iter.next();
            }
        }
    }
    while let Some(i_file) = index_file {
        if let Some(entry) = process_deleted_item(i_file) {
            file_changes.lock().unwrap().push(entry);
        }
        index_file = index_iter.next();
    }
}

fn process_deleted_item(index_entry: &DirEntry) -> Option<StatusEntry> {
    // When a submodule is missing it is *not* reported as deleted, it's assumed the user just
    // hasn't updated the submodules
    if index_entry.object_type == ObjectType::GitLink {
        return None;
    }
    Some(StatusEntry {
        name: index_entry.name.to_string(),
        state: Status::Deleted,
    })
}

fn is_modified(
    worktree_file: &mut jwalk::DirEntry<(IndexState, bool)>,
    index_file: &DirEntry,
    stats: &mut Option<DirectoryStat>,
) -> bool {
    if stats.is_none() {
        *stats = Some(DirectoryStat::new(worktree_file.parent_path()));
    }
    let dir_stat = stats.as_ref().unwrap();
    let name = worktree_file.file_name.to_str().unwrap().to_string();
    let stat = dir_stat.file_stats.get(&name).unwrap();
    let mut modified = false;
    if index_file.stat != *stat {
        modified = true;
    }
    modified
}

fn get_relative_entry_path_name(entry: &jwalk::DirEntry<(IndexState, bool)>) -> String {
    let path = entry.path();
    let root = path.ancestors().nth(entry.depth).unwrap();
    let relative_path = diff_paths(entry.path(), root).unwrap();
    relative_path.to_str().unwrap().replace("\\", "/")
}

fn process_new_item(
    dir_entry: &mut jwalk::DirEntry<(IndexState, bool)>,
    index: &Arc<Index>,
    ignores: &[Arc<Gitignore>],
) -> Option<StatusEntry> {
    let mut name = get_relative_entry_path_name(dir_entry);
    if dir_entry.file_type.is_dir() {
        if index.entries.contains_key(&name) {
            return None;
        }
        dir_entry.read_children_path = None;
    }

    if is_ignored(dir_entry, &name, ignores) {
        return None;
    }

    // Done after ignore as ignore doesn't handle trailing "/"
    if dir_entry.file_type.is_dir() {
        name.push('/');
    }

    Some(StatusEntry {
        name,
        state: Status::New,
    })
}

fn is_ignored(
    entry: &mut jwalk::DirEntry<(IndexState, bool)>,
    name: &str,
    ignores: &[Arc<Gitignore>],
) -> bool {
    let is_dir = entry.file_type.is_dir();
    for ignore in ignores {
        let matched = ignore.matched_path_or_any_parents(name, is_dir);

        // Whitelisting happens when a pattern is added back to valid files via the preceding "!"
        if matched.is_whitelist() {
            return false;
        }
        if matched.is_ignore() {
            return true;
        }
    }

    // For directories, we need to see if there are any files in the directory that
    // aren't ignored.
    if is_dir {
        let path = entry.path();
        let root = path.ancestors().nth(entry.depth).unwrap();
        return !directory_has_one_trackable_file(&root, &path, &ignores);
    }
    false
}

fn directory_has_one_trackable_file(root: &Path, dir: &Path, ignores: &[Arc<Gitignore>]) -> bool {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            let relative_path = diff_paths(&path, root).unwrap();
            let name = relative_path.to_str().unwrap().replace("\\", "/");
            let mut ignored = false;
            for ignore in ignores {
                let matched = ignore.matched_path_or_any_parents(&name, false);
                if matched.is_whitelist() {
                    ignored = false;
                    break;
                }
                if matched.is_ignore() {
                    ignored = true;
                    break;
                }
            }
            if !ignored {
                return true;
            }
        } else if directory_has_one_trackable_file(root, &path, ignores) {
            return true;
        }
    }
    false
}

fn submodule_status(
    dir_entry: &jwalk::DirEntry<(IndexState, bool)>,
    index_entry: &DirEntry,
    changed_files: &Arc<Mutex<Vec<StatusEntry>>>,
) {
    let name = get_relative_entry_path_name(dir_entry);
    let path = dir_entry.path();
    submodule_spawned_status(&name, &path, index_entry, changed_files);
}

fn submodule_spawned_status(
    name: &str,
    path: &Path,
    index_entry: &DirEntry,
    changed_files: &Arc<Mutex<Vec<StatusEntry>>>,
) {
    let repo = Repository::open(path).unwrap();
    let statuses = repo.statuses(None).unwrap();
    let mut modified_content = false;
    let mut untracked_content = false;
    for stat in statuses.iter() {
        if stat.head_to_index().is_some() {
            modified_content = true
        }
        if stat.index_to_workdir().is_some() {
            untracked_content = true
        }
    }
    let new_commits = index_entry.sha
        != repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .as_bytes();
    if modified_content || untracked_content || new_commits {
        changed_files.lock().unwrap().push(
            StatusEntry {
                name: name.to_string(),
                state: Status::Modified,
        });
    }
}

fn process_tracked_item(
    dir_entry: &mut jwalk::DirEntry<(IndexState, bool)>,
    index_entry: &DirEntry,
    stats: &mut Option<DirectoryStat>,
    changed_files: &Arc<Mutex<Vec<StatusEntry>>>,
) -> Option<StatusEntry> {
    if dir_entry.file_type.is_dir() {
        // Be sure and don't walk into submodules from here
        dir_entry.read_children_path = None;
        submodule_status(dir_entry, index_entry, changed_files);
        return None
    }

    if is_modified(dir_entry, index_entry, stats) {
        let name = get_relative_entry_path_name(dir_entry);
        return Some(StatusEntry {
            name,
            state: Status::Modified,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature, Time};
    use std::fs;
    use temp_testdir::TempDir;

    // Create a test repo to be able to compare the index to the working tree.
    pub fn test_repo(path: &Path, files: &Vec<&Path>) -> Index {
        let repo = Repository::init(path).unwrap();
        let mut index = repo.index().unwrap();
        let root = repo.path().parent().unwrap();
        for file in files {
            let full_path = root.join(file);

            // Done this way to support nested files
            fs::create_dir_all(full_path.parent().unwrap()).unwrap();
            fs::write(&full_path, file.to_str().unwrap()).unwrap();
            index.add_path(&file).unwrap();
        }
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
        repo.commit(
            Option::from("HEAD"),
            &signature,
            &signature,
            "A message",
            &tree,
            // No parents yet this is the first commit
            &[],
        )
        .unwrap();
        Index::new(&path.join(".git/index")).unwrap()
    }

    #[test]
    fn test_diff_against_index_nothing_modified() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);
        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        assert_eq!(value.entries, vec![]);
    }

    #[test]
    fn test_diff_against_index_a_file_modified_size() {
        let entry_name = "simple_file.txt";
        let temp_dir = TempDir::default();
        let mut index = test_repo(&temp_dir, &vec![Path::new(entry_name)]);
        let dir_entries = index.entries.get_mut("").unwrap();
        dir_entries[0].stat.size += 1;
        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: entry_name.to_string(),
            state: Status::Modified,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_diff_against_index_a_file_modified_mstat() {
        let entry_name = "simple_file.txt";
        let temp_dir = TempDir::default();
        let mut index = test_repo(&temp_dir, &vec![Path::new(entry_name)]);
        let dir_entries = index.entries.get_mut("").unwrap();
        dir_entries[0].stat.mtime += 1;
        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: entry_name.to_string(),
            state: Status::Modified,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_diff_against_index_deeply_nested() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("dir_1/dir_2/dir_3/file.txt")]);
        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        assert_eq!(value.entries, vec![]);
    }

    #[test]
    fn test_diff_against_modified_index_deeply_nested() {
        let temp_dir = TempDir::default();
        let mut index = test_repo(&temp_dir, &vec![Path::new("dir_1/dir_2/dir_3/file.txt")]);
        let dir_entries = index.entries.get_mut("dir_1/dir_2/dir_3").unwrap();
        dir_entries[0].stat.size += 1;
        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: "dir_1/dir_2/dir_3/file.txt".to_string(),
            state: Status::Modified,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_new_file_in_worktree() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);
        let new_file_name = "new_file.txt";
        let new_file = temp_dir.join(new_file_name);
        fs::create_dir_all(new_file.parent().unwrap()).unwrap();
        fs::write(&new_file, "stuff").unwrap();

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: new_file_name.to_string(),
            state: Status::New,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_multiple_new_files_in_worktree() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);

        // Putting them in order for the simpler assert
        let new_file_names = vec!["a_file.txt", "z_file.txt"];
        for name in &new_file_names {
            let new_file = temp_dir.join(&name);
            fs::create_dir_all(new_file.parent().unwrap()).unwrap();
            fs::write(&new_file, "stuff").unwrap();
        }

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries: Vec<StatusEntry> = new_file_names
            .iter()
            .map(|&n| StatusEntry {
                name: n.to_string(),
                state: Status::New,
            })
            .collect();
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_new_directory_in_worktree_does_not_show() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);
        fs::create_dir_all(temp_dir.join("new_dir")).unwrap();

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        assert_eq!(value.entries, vec![]);
    }

    #[test]
    fn test_deleted_file_in_worktree() {
        let names = vec!["file_1.txt", "file_2.txt", "foo.txt"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &files);
        fs::remove_file(temp_dir.join("file_2.txt")).unwrap();

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: "file_2.txt".to_string(),
            state: Status::Deleted,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_deleted_file_at_end_of_worktree() {
        let names = vec!["file_1.txt", "file_2.txt", "foo.txt"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &files);
        fs::remove_file(temp_dir.join("foo.txt")).unwrap();

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();
        let entries = vec![StatusEntry {
            name: "foo.txt".to_string(),
            state: Status::Deleted,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_ignored_file_in_worktree() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);

        for name in vec!["ignored.txt", ".gitignore"] {
            let file = temp_dir.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "ignore*").unwrap();
        }

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();

        // Only the gitignore should show up as new
        let entries = vec![StatusEntry {
            name: ".gitignore".to_string(),
            state: Status::New,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_ignored_directory_in_worktree() {
        let temp_dir = TempDir::default();
        let index = test_repo(&temp_dir, &vec![Path::new("simple_file.txt")]);

        for name in vec!["foo/ignored.txt", ".gitignore"] {
            let file = temp_dir.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "foo/").unwrap();
        }

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();

        // Only the gitignore should show up as new
        let entries = vec![StatusEntry {
            name: ".gitignore".to_string(),
            state: Status::New,
        }];
        assert_eq!(value.entries, entries);
    }

    #[test]
    fn test_unignored_files() {
        let seed_names = vec!["simple_file.txt", "foo/.gitignore"];
        let temp_dir = TempDir::default();
        let files = seed_names.iter().map(|n| Path::new(n)).collect();
        let index = test_repo(&temp_dir, &files);

        for name in vec![
            "foo/ignored.txt",
            ".gitignore",
            "bar/always.txt",
            "foo/what/why/ignored.txt",
        ] {
            let file = temp_dir.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "ignore*\nalways*").unwrap();
        }
        let file = temp_dir.join("foo/.gitignore");
        fs::write(&file, "!ignore*").unwrap();

        let value = WorkTree::diff_against_index(&temp_dir, index).unwrap();

        let entries = vec![
            StatusEntry {
                name: ".gitignore".to_string(),
                state: Status::New,
            },
            StatusEntry {
                name: "foo/.gitignore".to_string(),
                state: Status::Modified,
            },
            StatusEntry {
                name: "foo/ignored.txt".to_string(),
                state: Status::New,
            },
            StatusEntry {
                name: "foo/what/".to_string(),
                state: Status::New,
            },
        ];
        assert_eq!(value.entries, entries);
    }
}
