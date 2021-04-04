/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::{WorkTree, TreeDiff, Index};
use std::path::Path;
use crate::error::StatusError;

#[derive(Debug)]
pub struct RepoStatus {
    index_diff: TreeDiff,
    work_tree_diff: WorkTree,
}

impl RepoStatus {
    pub(crate) fn branch_name(&self) -> String {
        "toasty".to_string()
    }
}

impl RepoStatus {
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<RepoStatus, StatusError> {
        let index_file = path.join(".git/index");
        let index = Index::new(&*index_file)?;
        let work_tree_diff = WorkTree::diff_against_index(path, index)?;
        let index_diff = TreeDiff::diff_against_index(path);
        Ok(RepoStatus { index_diff, work_tree_diff })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature, Time};
    use std::fs;
    use temp_testdir::TempDir;

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
    fn test_branch_name() {
        let temp_dir = TempDir::default();
        test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let status = RepoStatus::new(&temp_dir).unwrap();
        assert_eq!(status.branch_name(), "toasty");
    }
}