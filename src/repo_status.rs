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
        let repo_path = repo.path();
        let index_file = repo_path.join("index");
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
        let name = self.branch_name().unwrap();
        let short_name = name.strip_prefix("refs/heads/").unwrap();
        let branch_name = "On branch ".to_string() + &short_name;
        branch_name
    }

    fn branch_name(&self) -> Option<String> {
        let branch = self.repo.head().unwrap();
        let name = branch.name();
        match name {
            None => None,
            Some("HEAD") => None,
            Some(branch_name) => Some(branch_name.to_string()),
        }
    }
    fn get_remote_branch_difference_message(&self) -> String {
        let name = self
            .repo
            .branch_upstream_name(&self.branch_name().unwrap())
            .unwrap();
        let short_name = name
            .as_str()
            .unwrap()
            .strip_prefix("refs/remotes/")
            .unwrap();

        let head = self.repo.head().unwrap();
        let head_commit = head.peel_to_commit().unwrap();
        let local_oid = head_commit.id();
        let upstream_ref = self.repo.find_reference(name.as_str().unwrap()).unwrap();
        let upstream_commit = upstream_ref.peel_to_commit().unwrap();
        let upstream_oid = upstream_commit.id();
        let (before, after) = self.repo.graph_ahead_behind(local_oid, upstream_oid).unwrap();

        match before {
            0 => match after {
                0 => "Your branch is up to date with '".to_string() + &short_name.to_string() + &"'.".to_string(),
                _ => "What".to_string(),
            },
            _ => "why".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Commit, Repository, Signature, Time, BranchType};
    use std::fs;
    use temp_testdir::TempDir;
    use indoc::indoc;

    // A test repo to be able to test message state generation.  This repo will have 2 branches
    // created:
    //  * `tip`: This will be the same as `main` or `master`.  Due to current inconsistencies
    //           with the new default name for the first branch `tip` was chosen.
    //  * `half`: This will be half way between the `tip` and repo start.
    //
    // There will also be a remote repo with branches `tip` and `half`.  The remote repo will be
    // in a sibling directory of the returned repo at the directory named `repo`.
    pub fn test_repo(path: &str, files: &Vec<&Path>) -> Repository {
        let remote_path = path.to_string() + "/remote";
        let remote_repo = Repository::init(Path::new(&remote_path)).unwrap();
        for file in files {
            commit_file(&remote_repo, file)
        }
        create_branches(&remote_repo);

        let main_path = path.to_string() + "/main";
        let repo = Repository::clone(&remote_path, &main_path).unwrap();
        for branch_name in vec!["tip", "half"] {
            let upstream_name = "origin/".to_string() + branch_name;
            let remote_branch_name = "refs/remotes/".to_string() + &upstream_name;
            repo.set_head(&remote_branch_name).unwrap();
            let commit = repo.head().unwrap().peel_to_commit().unwrap();
            let mut branch = repo.branch(&branch_name, &commit, false).unwrap();
            branch.set_upstream(Some(&upstream_name)).unwrap();
        }
        repo.set_head("refs/heads/tip").unwrap();
        repo
    }

    fn create_branches(repo: &Repository) {
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("tip", &commit, false).unwrap();
        repo.branch("half", &commit, false).unwrap();
    }

    fn commit_file(repo: &Repository, file: &Path) {
        let mut index = repo.index().unwrap();
        let root = repo.path().parent().unwrap();
        let full_path = root.join(file);

        // Done this way to support nested files
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, file.to_str().unwrap()).unwrap();
        index.add_path(&file).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
        let head = repo.head();
        let _parents = match head {
            Err(_) => vec![],
            _ => vec![head.unwrap().peel_to_commit().unwrap()],
        };
        let parents: Vec<&Commit> = _parents.iter().map(|n| n).collect();
        let message = "Commiting file: ".to_string() + file.to_str().unwrap();
        repo.commit(
            Option::from("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &parents[..],
        )
        .unwrap();
    }

    #[test]
    fn test_branch_name() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("why_not")]);
        repo.set_head("refs/heads/tip").unwrap();
        let status = RepoStatus::new(repo.path()).unwrap();
        assert_eq!(status.branch_name().unwrap(), "refs/heads/tip");
    }

    #[test]
    fn test_new_branch_name() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("why_not")]);
        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        assert_eq!(status.branch_name().unwrap(), "refs/heads/half");
    }

    #[test]
    fn test_detached_branch_state() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("why_not")]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();

        repo.set_head_detached(commit.as_object().id()).unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        assert_eq!(status.branch_name(), None);
    }

    #[test]
    fn test_get_branch_message_normal() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("a_file")]);

        repo.set_head("refs/heads/tip").unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch tip");
    }

    #[test]
    fn test_get_branch_message_different_branch() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("i_guess")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch half");
    }

    #[test]
    fn test_get_remote_branch_difference_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);
        let status = RepoStatus::new(repo.path()).unwrap();
        let message = status.get_remote_branch_difference_message();
        assert_eq!(message, "Your branch is up to date with 'origin/tip'.");
    }

    #[test]
    fn test_get_remote_branch_different_remote_name_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        let message = status.get_remote_branch_difference_message();
        assert_eq!(message, "Your branch is up to date with 'origin/half'.");
    }

    #[test]
    fn test_behind_remote_branch() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        repo.set_head("refs/heads/half").unwrap();
        let mut branch = repo.find_branch("half", BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/tip")).unwrap();

        let status = RepoStatus::new(repo.path()).unwrap();
        let message = status.get_remote_branch_difference_message();
        let expected = indoc!{"\
            Your branch is behind 'origin/tip' by 2 commits, and can be fast-forwarded.
              (use \"git pull\" to update your local branch)
        "};
        assert_eq!(message, expected);
    }
}
