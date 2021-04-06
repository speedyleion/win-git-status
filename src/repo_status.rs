/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::error::StatusError;
use crate::{Index, TreeDiff, WorkTree};
use git2::Repository;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::Path;

pub struct RepoStatus {
    repo: Repository,
    index_diff: TreeDiff,
    work_tree_diff: WorkTree,
}

impl Debug for RepoStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}\n{:?}", self.index_diff, self.work_tree_diff)
    }
}

impl RepoStatus {
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<RepoStatus, StatusError> {
        let repo = Repository::open(path).unwrap();
        let index_file = path.join(".git/index");
        let index = Index::new(&*index_file)?;
        let work_tree_diff = WorkTree::diff_against_index(path, index)?;
        let index_diff = TreeDiff::diff_against_index(path);
        Ok(RepoStatus {
            repo,
            index_diff,
            work_tree_diff,
        })
    }

    pub(crate) fn get_branch_message(&self) -> String {
        let branch_name = "On branch ".to_string() + &self.branch_name().unwrap();
        branch_name
    }

    fn branch_name(&self) -> Option<String> {
        let branch = self.repo.head().unwrap();
        let name = branch.name();
        match name {
            None => None,
            Some(branch_name) => Some(branch_name.to_string()),
        }
    }
    fn get_remote_branch_difference_message(&self) -> String {
        let name = self.repo.branch_upstream_name(&self.branch_name().unwrap()).unwrap();
        let short_name = name.as_str().unwrap().strip_prefix("refs/remotes/").unwrap();
        "Your branch is up to date with '".to_string() + &short_name.to_string() + &"'".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature, Time};
    use std::fs;
    use temp_testdir::TempDir;

    // Create a test repo to be able to compare the index to the working tree.
    pub fn test_repo(path: &str, files: &Vec<&Path>) -> Repository {
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
        {
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
        repo
    }

    #[test]
    fn test_branch_name() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("a_branch", &commit, false).unwrap();

        repo.set_head("refs/heads/a_branch").unwrap();
        let status = RepoStatus::new(&temp_dir).unwrap();
        assert_eq!(status.branch_name().unwrap(), "a_branch");
    }

    #[test]
    fn test_new_branch_name() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("new_branch", &commit, false).unwrap();

        repo.set_head("refs/heads/new_branch").unwrap();

        let status = RepoStatus::new(&temp_dir).unwrap();
        assert_eq!(status.branch_name().unwrap(), "new_branch");
    }

    #[test]
    fn test_detached_branch_state() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();

        repo.set_head_detached(commit.as_object().id()).unwrap();

        let status = RepoStatus::new(&temp_dir).unwrap();
        assert_eq!(status.branch_name(), None);
    }

    #[test]
    fn test_get_branch_message_normal() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("new_branch", &commit, false).unwrap();

        repo.set_head("refs/heads/new_branch").unwrap();

        let status = RepoStatus::new(&temp_dir).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch new_branch");
    }

    #[test]
    fn test_get_branch_message_different_branch() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("what", &commit, false).unwrap();

        repo.set_head("refs/heads/what").unwrap();

        let status = RepoStatus::new(&temp_dir).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch what");
    }

    #[test]
    fn test_get_remote_branch_difference_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("what", &commit, false).unwrap();

        repo.set_head("refs/heads/what").unwrap();

        let status = RepoStatus::new(&temp_dir).unwrap();
        let message = status.get_remote_branch_difference_message();
        assert_eq!(message, "Your branch is up to date with 'origin/what'.");
    }

    #[test]
    fn test_get_remote_branch_different_remote_name_same_spot() {
        let temp_dir = TempDir::default();
        let remote_dir = temp_dir.join("remote");
        let remote = test_repo(remote_dir.to_str().unwrap(), &vec![]);
        let commit = remote.head().unwrap().peel_to_commit().unwrap();
        remote.branch("sure", &commit, false).unwrap();

        let main_dir = temp_dir.join("main");
        let main_repo = Repository::clone(remote_dir.to_str().unwrap(), &main_dir).unwrap();
        let commit = main_repo.head().unwrap().peel_to_commit().unwrap();
        let mut branch = main_repo.branch("sure", &commit, false).unwrap();
        branch.set_upstream(Some("origin/sure")).unwrap();
        main_repo.set_head("refs/heads/sure").unwrap();

        let status = RepoStatus::new(&main_dir).unwrap();
        let message = status.get_remote_branch_difference_message();
        assert_eq!(message, "Your branch is up to date with 'origin/sure'");
    }
}
