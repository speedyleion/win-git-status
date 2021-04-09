/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::error::StatusError;
use crate::{Index, TreeDiff, WorkTree};
use git2::Repository;
use indoc::formatdoc;
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

    pub fn message(&self) -> Result<String, StatusError> {
        let branch = self.get_branch_message();
        let remote_state = self.get_remote_branch_difference_message();
        let unstaged = match self.get_unstaged_message() {
            None => "".to_string(),
            Some(message) => message,
        };
        Ok(formatdoc! {"\
           {branch}
           {remote_state}
           {unstaged}", branch=branch, remote_state=remote_state, unstaged=unstaged})
    }

    pub fn get_branch_message(&self) -> String {
        let name = match self.branch_name() {
            Some(name) => name,
            None => return self.get_detached_message(),
        };
        let short_name = name.strip_prefix("refs/heads/").unwrap();
        let message = "On branch ".to_string();
        message + short_name
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
        let (before, after) = self
            .repo
            .graph_ahead_behind(local_oid, upstream_oid)
            .unwrap();

        match before {
            0 => match after {
                0 => {
                    formatdoc! {"\
                    Your branch is up to date with '{branch}'.",
                    branch=short_name }
                }
                _ => {
                    let plural = match after {
                        1 => "",
                        _ => "s",
                    };
                    formatdoc! {"\
                        Your branch is behind '{branch}' by {commits} commit{plural}, and can be fast-forwarded.
                          (use \"git pull\" to update your local branch)",
                    branch=short_name, commits=after, plural=plural }
                }
            },
            _ => match after {
                0 => {
                    let plural = match before {
                        1 => "",
                        _ => "s",
                    };
                    formatdoc! {"\
                        Your branch is ahead of '{branch}' by {commits} commit{plural}.
                          (use \"git push\" to publish your local commits)",
                    branch=short_name, commits=before, plural=plural
                    }
                }
                _ => {
                    formatdoc! {"\
                        Your branch and '{branch}' have diverged,
                        and have {before} and {after} different commits each, respectively.
                          (use \"git pull\" to merge the remote branch into yours)",
                    branch=short_name, before=before, after=after
                    }
                }
            },
        }
    }
    fn get_detached_message(&self) -> String {
        let commit_sha = self
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();
        let short_sha = &commit_sha[..7];
        formatdoc! {
            "Head detached at {sha}",
            sha=short_sha
        }
    }
    fn get_unstaged_message(&self) -> Option<String> {
        let unstaged_files: Vec<String> = self
            .work_tree_diff
            .entries
            .iter()
            .map(|e| e.to_string())
            .collect();
        if unstaged_files.is_empty() {
            return None;
        }
        let files = unstaged_files
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("\n        ");
        let message = formatdoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    {files}",
        files=files};
        Some(message)
    }
    fn get_staged_message(&self) -> Option<String> {
        let staged_files: Vec<String> = self
            .index_diff
            .entries
            .iter()
            .map(|e| e.to_string())
            .collect();
        let files = staged_files
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("\n        ");
        let message = formatdoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    {files}", files=files};
        Some(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{BranchType, Commit, Repository, Signature, Time};
    use indoc::indoc;
    use std::fs;
    use temp_testdir::TempDir;

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
        let mut walker = repo.revwalk().unwrap();
        walker.push(commit.id()).unwrap();
        let count = walker.count();
        let half = count / 2;
        let mut walker = repo.revwalk().unwrap();
        walker.push(commit.id()).unwrap();
        let half_oid = walker.skip(half).next().unwrap().unwrap();
        let half_commit = repo.find_commit(half_oid).unwrap();
        repo.branch("half", &half_commit, false).unwrap();
    }

    fn write_to_file(repo: &Repository, file: &Path, contents: &str) {
        let root = repo.workdir().unwrap();
        let full_path = root.join(file);

        // Done this way to support nested files
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, contents).unwrap();
    }

    fn stage_file(repo: &Repository, file: &Path) {
        let mut index = repo.index().unwrap();
        index.add_path(&file).unwrap();
        index.write().unwrap();
    }

    fn commit_file(repo: &Repository, file: &Path) {
        write_to_file(repo, file, file.to_str().unwrap());
        stage_file(repo, file);
        let mut index = repo.index().unwrap();
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
        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        assert_eq!(status.branch_name().unwrap(), "refs/heads/tip");
    }

    #[test]
    fn test_new_branch_name() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("why_not")]);
        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        assert_eq!(status.branch_name().unwrap(), "refs/heads/half");
    }

    #[test]
    fn test_detached_branch_state() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("why_not")]);
        let commit = repo.head().unwrap().peel_to_commit().unwrap();

        repo.set_head_detached(commit.as_object().id()).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        assert_eq!(status.branch_name(), None);
    }

    #[test]
    fn test_get_branch_message_normal() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("a_file")]);

        repo.set_head("refs/heads/tip").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch tip");
    }

    #[test]
    fn test_get_branch_message_different_branch() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("i_guess")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "On branch half");
    }

    #[test]
    fn test_get_branch_message_detached_parent() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let parent = head.parent(0).unwrap().id();
        repo.set_head_detached(parent).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "Head detached at 17fe299");
    }

    #[test]
    fn test_get_branch_message_detached_tip() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.set_head_detached(head.id()).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_branch_message();
        assert_eq!(message, "Head detached at 82578fa");
    }

    #[test]
    fn test_get_remote_branch_difference_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);
        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_remote_branch_difference_message();
        assert_eq!(message, "Your branch is up to date with 'origin/tip'.");
    }

    #[test]
    fn test_get_remote_branch_different_remote_name_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
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

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_remote_branch_difference_message();
        let expected = indoc! {"\
            Your branch is behind 'origin/tip' by 2 commits, and can be fast-forwarded.
              (use \"git pull\" to update your local branch)"};
        assert_eq!(message, expected);
    }

    #[test]
    fn test_ahead_of_remote_branch_by_one() {
        let file_names = vec!["one", "two"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        repo.set_head("refs/heads/tip").unwrap();
        let mut branch = repo.find_branch("tip", BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/half")).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_remote_branch_difference_message();
        let expected = indoc! {"\
            Your branch is ahead of 'origin/half' by 1 commit.
              (use \"git push\" to publish your local commits)"};
        assert_eq!(message, expected);
    }

    #[test]
    fn test_ahead_of_remote_branch_by_three() {
        let file_names = vec!["one", "two", "three", "four", "five", "six"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        repo.set_head("refs/heads/tip").unwrap();
        let mut branch = repo.find_branch("tip", BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/half")).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_remote_branch_difference_message();
        let expected = indoc! {"\
            Your branch is ahead of 'origin/half' by 3 commits.
              (use \"git push\" to publish your local commits)"};
        assert_eq!(message, expected);
    }

    #[test]
    fn test_diverged_from_remote_branch() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        repo.set_head("refs/heads/half").unwrap();
        let mut branch = repo.find_branch("half", BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/tip")).unwrap();

        let new_files = vec!["what", "why", "where"];
        for file in new_files {
            commit_file(&repo, Path::new(file));
        }

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_remote_branch_difference_message();
        let expected = indoc! {"\
            Your branch and 'origin/tip' have diverged,
            and have 3 and 2 different commits each, respectively.
              (use \"git pull\" to merge the remote branch into yours)"};
        assert_eq!(message, expected);
    }

    #[test]
    fn test_no_modified_files() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_unstaged_message();
        assert_eq!(message, None);
    }

    #[test]
    fn test_one_modified_file() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[2], "what???");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_unstaged_message();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   three"};
        assert_eq!(message, Some(expected.to_string()));
    }

    #[test]
    fn test_two_modified_files() {
        let file_names = vec!["one/nested/a/bit.txt", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[0], "what???");
        write_to_file(&repo, files[3], "what???");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_unstaged_message();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   four
                    modified:   one/nested/a/bit.txt"};
        assert_eq!(message, Some(expected.to_string()));
    }

    #[test]
    fn test_two_modified_one_deleted() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[0], "what???");
        let workdir = repo.workdir().unwrap();
        fs::remove_file(workdir.join(files[2])).unwrap();
        write_to_file(&repo, files[1], "what???");

        let status = RepoStatus::new(workdir).unwrap();
        let message = status.get_unstaged_message();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   one
                    deleted:    three
                    modified:   two"};
        assert_eq!(message, Some(expected.to_string()));
    }

    #[test]
    fn test_file_added_to_index() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let new_file = Path::new("a_new_file");
        write_to_file(&repo, new_file, "stuff");
        stage_file(&repo, new_file);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_staged_message();

        let expected = indoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    new file:   a_new_file"};
        assert_eq!(message, Some(expected.to_string()));
    }

    #[test]
    fn test_file_added_to_index_and_files_modified() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let new_file = Path::new("some_new_file");
        write_to_file(&repo, new_file, "stuff");
        stage_file(&repo, new_file);

        for file in vec![files[0], files[1]] {
            write_to_file(&repo, file, "what???");
            stage_file(&repo, file);
        }

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let message = status.get_staged_message();

        let expected = indoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    modified:   one
                    new file:   some_new_file
                    modified:   two"};
        assert_eq!(message, Some(expected.to_string()));
    }
}
