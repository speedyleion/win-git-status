/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use git2::{Repository, StatusOptions, StatusShow, Statuses};
use std::path::Path;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    CURRENT,
    NEW,
    MODIFIED,
    DELETED,
}

impl Default for Status {
    fn default() -> Self {
        Status::CURRENT
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TreeDiffEntry {
    pub name: String,
    pub state: Status,
}

/// A tree of a repo.
///
#[derive(Debug, Default, PartialEq)]
pub struct TreeDiff {
    pub entries: Vec<TreeDiffEntry>,
}

impl TreeDiff {
    pub fn diff_against_index(path: &Path) -> TreeDiff {
        let repo = Repository::open(path).unwrap();
        let mut options = StatusOptions::new();
        options.show(StatusShow::Index);
        let diff = repo.statuses(Option::from(&mut options)).unwrap();
        TreeDiff::convert_git2_to_treediff(&diff)
    }

    fn convert_git2_to_treediff(statuses: &Statuses) -> TreeDiff {
        let mut entries = vec![];
        for status in statuses.iter() {
            let state = TreeDiff::git2_status_to_treediff_status(status.status());
            entries.push(TreeDiffEntry {
                name: status.path().unwrap().to_string(),
                state,
            });
        }
        TreeDiff { entries }
    }
    fn git2_status_to_treediff_status(status: git2::Status) -> Status {
        match status {
            git2::Status::INDEX_NEW => Status::NEW,
            git2::Status::INDEX_MODIFIED => Status::MODIFIED,
            git2::Status::INDEX_DELETED => Status::DELETED,
            _ => panic!("Unsupported index status {:?}", status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Signature, Time};
    use std::fs;
    use temp_testdir::TempDir;

    // stage a file change so that the index version of a file differs from a tree version.
    pub fn stage_file(repo_path: &str, file: &Path) -> () {
        let repo = Repository::open(repo_path).unwrap();
        let mut index = repo.index().unwrap();
        let root = repo.path().parent().unwrap();
        let full_path = root.join(file);

        fs::write(&full_path, "staged changes").unwrap();
        index.add_path(&file).unwrap();
        index.write().unwrap();
    }

    // Create a test repo to be able to compare the index to the working tree.
    pub fn test_repo(path: &str, files: &Vec<&Path>) -> () {
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
    }

    #[test]
    fn test_get_tree_diff_empty_repo() {
        let temp_dir = TempDir::default();
        test_repo(temp_dir.to_str().unwrap(), &vec![]);
        assert_eq!(TreeDiff::diff_against_index(&temp_dir), TreeDiff::default());
    }

    #[test]
    fn test_get_tree_diff_1_file_changed() {
        let names = vec!["one.baz"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo_path = temp_dir.to_str().unwrap();
        test_repo(repo_path, &files);

        stage_file(repo_path, files[0]);
        let diff = TreeDiff::diff_against_index(&temp_dir);
        assert_eq!(
            diff,
            TreeDiff {
                entries: vec![TreeDiffEntry {
                    name: names[0].to_string(),
                    state: Status::MODIFIED
                }]
            }
        );
    }
}
