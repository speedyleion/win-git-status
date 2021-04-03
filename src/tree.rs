/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */


use crate::{DirEntry, Index};
use std::path::Path;
use git2::{Repository, TreeWalkMode, TreeWalkResult, TreeEntry, ObjectType};
use crate::direntry::FileStat;
use std::convert::TryInto;

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
    path: String,
    pub entries: Vec<TreeDiffEntry>,
}

impl TreeDiff {
    pub fn diff_against_index(path: &Path) -> TreeDiff {
        TreeDiff::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;
    use temp_testdir::TempDir;
    use git2::{Signature, Time};
    use std::fs;

    // Create a test repo to be able to compare the index to the working tree.
    pub fn test_repo(path: &str, files: Vec<&Path>) -> () {
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
        let repo = test_repo(temp_dir.to_str().unwrap(), vec![]);
        assert_eq!(TreeDiff::diff_against_index(&temp_dir), TreeDiff::default());
    }

    #[test]
    fn test_get_tree_diff_1_file_changed() {
        let mut names = vec!["one.baz"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), files);

        let diff = TreeDiff::diff_against_index(&temp_dir);
        assert_eq!(diff.entries.len(), 1);
        let diff_names: Vec<&String> = diff.entries.iter().map(|e| &e.name).collect();
        names.sort();
        assert_eq!(diff_names, names);
    }

}
