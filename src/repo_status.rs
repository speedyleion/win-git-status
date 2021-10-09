/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::error::StatusError;
use crate::status::{Status, StatusEntry};
use crate::{Index, TreeDiff, WorkTree};
use git2::{Oid, Repository, RepositoryState};
use indoc::formatdoc;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::path::Path;
use termcolor::{Color, ColorSpec, WriteColor};

// See for the list of slots https://git-scm.com/docs/git-config#Documentation/git-config.txt-colorstatusltslotgt
enum StatusColorSlot {
    Untracked,
    Changed,
    Added,
    NoBranch,
}

impl fmt::Display for StatusColorSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusColorSlot::Untracked => write!(f, "untracked"),
            StatusColorSlot::Changed => write!(f, "changed"),
            StatusColorSlot::Added => write!(f, "added"),
            StatusColorSlot::NoBranch => write!(f, "nobranch"),
        }
    }
}

impl StatusColorSlot {
    pub fn default_color(&self) -> Color {
        match self {
            StatusColorSlot::Untracked => Color::Red,
            StatusColorSlot::Changed => Color::Red,
            StatusColorSlot::Added => Color::Green,
            StatusColorSlot::NoBranch => Color::Red,
        }
    }
}

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

impl From<git2::Error> for StatusError {
    fn from(err: git2::Error) -> StatusError {
        StatusError {
            message: err.to_string(),
        }
    }
}

impl RepoStatus {
    /// * `path` - The path to a git repo.  This logic will search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<RepoStatus, StatusError> {
        let repo: Repository;
        let discovery = Repository::discover(path);
        match discovery {
            // Developer note:
            // The behaviour for when this is an error is not tested in an automated fashion.
            // Due to different machines, their test environments, and where git repos could exist
            // it was decided to avoid misleading test failures and this was tested at one point
            // manually.
            Err(_e) => {
                return Err(StatusError {
                    message: "fatal: not a git repository (or any of the parent directories): .git"
                        .to_string(),
                })
            }
            Ok(r) => repo = r,
        };
        let repo_path = repo.path();
        let index_file = repo_path.join("index");
        let index = Index::new(&*index_file)?;
        let workdir = repo.workdir().unwrap();
        let (work_tree_diff, index_diff) = rayon::join(
            || WorkTree::diff_against_index(workdir, index).unwrap(),
            || TreeDiff::diff_against_index(path),
        );
        Ok(RepoStatus {
            repo,
            index_diff,
            work_tree_diff,
        })
    }

    pub fn write_short_message<W: WriteColor + Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StatusError> {
        self.check_repo_state()?;
        self.write_short_staged(writer);
        self.write_short_unstaged(writer);
        self.write_short_untracked(writer);
        Ok(())
    }

    pub fn write_long_message<W: WriteColor + Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StatusError> {
        self.check_repo_state()?;
        self.write_branch_message(writer)?;
        self.write_remote_branch_difference_message(writer);
        let staged = self.write_staged_message(writer);
        let unstaged = self.write_unstaged_message(writer);
        let untracked = self.write_untracked_message(writer);
        RepoStatus::write_epilog(writer, staged, unstaged, untracked);
        Ok(())
    }

    fn get_color(&self, color_slot: StatusColorSlot) -> Color {
        let config = self.repo.config().unwrap();
        let config_string = format!("color.status.{}", color_slot);
        let color = config.get_string(&config_string);
        match color {
            Ok(color) => match color.parse() {
                Ok(color) => color,
                Err(_) => color_slot.default_color(),
            },
            Err(_) => color_slot.default_color(),
        }
    }

    fn write_branch_message<W: WriteColor + Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), StatusError> {
        let name = match self.branch_name() {
            Some(name) => name,
            None => {
                self.get_detached_message(writer);
                return Ok(());
            }
        };
        let short_name = name.strip_prefix("refs/heads/").unwrap();
        let message = format! {"On branch {}\n", short_name};
        writer.write_all(message.as_bytes())?;
        Ok(())
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

    fn write_remote_branch_difference_message<W: WriteColor + Write>(&self, writer: &mut W) {
        let name = match self.upstream_branch_name() {
            Some(name) => name,
            None => return,
        };

        let short_name = name.strip_prefix("refs/remotes/").unwrap();

        let upstream_oid = match self.get_oid(&name) {
            Some(oid) => oid,
            None => {
                let message = formatdoc! {"\
                    Your branch is based on '{branch}', but the upstream is gone.
                     (use \"git branch --unset-upstream\" to fixup)", branch=short_name};
                writer.write_all(message.as_bytes()).unwrap();
                writer.write_all(b"\n\n").unwrap();
                return;
            }
        };

        let head = self.repo.head().unwrap();
        let head_commit = head.peel_to_commit().unwrap();
        let local_oid = head_commit.id();

        let (before, after) = self
            .repo
            .graph_ahead_behind(local_oid, upstream_oid)
            .unwrap();

        let message: String;
        match before {
            0 => match after {
                0 => {
                    message = formatdoc! {"\
                    Your branch is up to date with '{branch}'.",
                    branch=short_name }
                }
                _ => {
                    let plural = match after {
                        1 => "",
                        _ => "s",
                    };
                    message = formatdoc! {"\
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
                    message = formatdoc! {"\
                        Your branch is ahead of '{branch}' by {commits} commit{plural}.
                          (use \"git push\" to publish your local commits)",
                    branch=short_name, commits=before, plural=plural
                    }
                }
                _ => {
                    message = formatdoc! {"\
                        Your branch and '{branch}' have diverged,
                        and have {before} and {after} different commits each, respectively.
                          (use \"git pull\" to merge the remote branch into yours)",
                    branch=short_name, before=before, after=after
                    }
                }
            },
        }
        writer.write_all(message.as_bytes()).unwrap();
        writer.write_all(b"\n\n").unwrap();
    }

    fn upstream_branch_name(&self) -> Option<String> {
        match self.branch_name() {
            None => None,
            Some(branch_name) => match self.repo.branch_upstream_name(&branch_name) {
                Err(_e) => None,
                Ok(name) => Some(name.as_str().unwrap().to_string()),
            },
        }
    }

    fn get_detached_message<W: WriteColor + Write>(&self, writer: &mut W) {
        let commit_sha = self
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();
        let short_sha = &commit_sha[..7];
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(self.get_color(StatusColorSlot::NoBranch)));
        writer.set_color(&color_spec).unwrap();
        writer.write_all(b"Head detached at ").unwrap();
        writer.reset().unwrap();
        writer.write_all(short_sha.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }

    fn write_unstaged_message<W: WriteColor + Write>(&self, writer: &mut W) -> bool {
        let unstaged_files: Vec<&StatusEntry> = self
            .work_tree_diff
            .entries
            .iter()
            .filter(|e| e.state != Status::New)
            .collect();
        if unstaged_files.is_empty() {
            return false;
        }
        let message = formatdoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)"};

        writer.write_all(message.as_bytes()).unwrap();

        let mut color_spec = ColorSpec::new();
        let changed_color = Some(self.get_color(StatusColorSlot::Changed));
        color_spec.set_fg(changed_color);
        writer.set_color(&color_spec).unwrap();
        for file in unstaged_files {
            let modified_line = format! {"\n        {}", file.to_string()};
            writer.write_all(modified_line.as_bytes()).unwrap();
            if let Status::Modified(Some(message)) = &file.state {
                writer.reset().unwrap();
                let info = format! {" ({})", message};
                writer.write_all(info.as_bytes()).unwrap();
                writer.set_color(&color_spec).unwrap();
            }
        }
        writer.reset().unwrap();
        writer.write_all(b"\n\n").unwrap();
        true
    }

    fn write_staged_message<W: WriteColor + Write>(&self, writer: &mut W) -> bool {
        let staged_files: Vec<String> = self
            .index_diff
            .entries
            .iter()
            .map(|e| e.to_string())
            .collect();

        if staged_files.is_empty() {
            return false;
        }

        let files = staged_files
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("\n        ");
        let message = formatdoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    "};

        writer.write_all(message.as_bytes()).unwrap();

        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(self.get_color(StatusColorSlot::Added)));
        writer.set_color(&color_spec).unwrap();
        writer.write_all(files.as_bytes()).unwrap();
        writer.reset().unwrap();
        writer.write_all(b"\n\n").unwrap();
        true
    }

    fn write_untracked_message<W: WriteColor + Write>(&self, writer: &mut W) -> bool {
        let untracked_files: Vec<String> = self
            .work_tree_diff
            .entries
            .iter()
            .filter(|e| e.state == Status::New)
            .map(|e| e.name.to_string())
            .collect();

        if untracked_files.is_empty() {
            return false;
        }

        let files = untracked_files
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("\n        ");

        let message = formatdoc! {"\
            Untracked files:
              (use \"git add <file>...\" to include in what will be committed)
                    "};
        writer.write_all(message.as_bytes()).unwrap();

        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(self.get_color(StatusColorSlot::Untracked)));
        writer.set_color(&color_spec).unwrap();
        writer.write_all(files.as_bytes()).unwrap();
        writer.reset().unwrap();
        writer.write_all(b"\n\n").unwrap();
        true
    }

    fn write_epilog<W: WriteColor + Write>(
        writer: &mut W,
        staged: bool,
        unstaged: bool,
        untracked: bool,
    ) {
        if staged {
            return;
        }
        if unstaged {
            writer
                .write_all(
                    b"no changes added to commit (use \"git add\" and/or \"git commit -a\")\n",
                )
                .unwrap();
            return;
        }
        if untracked {
            writer.write_all(b"nothing added to commit but untracked files present (use \"git add\" to track)\n").unwrap();
            return;
        }
        writer
            .write_all(b"nothing to commit, working tree clean\n")
            .unwrap();
    }

    fn check_repo_state(&self) -> Result<(), StatusError> {
        let state = self.repo.state();
        let unsupported_state = match state {
            RepositoryState::Clean => return Ok(()),
            RepositoryState::Merge => "Merge",
            RepositoryState::Revert => "Revert",
            RepositoryState::RevertSequence => "RevertSequence",
            RepositoryState::CherryPick => "CherryPick",
            RepositoryState::CherryPickSequence => "CherryPickSequence",
            RepositoryState::Bisect => "Bisect",
            RepositoryState::Rebase => "Rebase",
            RepositoryState::RebaseInteractive => "RebaseInteractive",
            RepositoryState::RebaseMerge => "RebaseMerge",
            RepositoryState::ApplyMailbox => "ApplyMailbox",
            RepositoryState::ApplyMailboxOrRebase => "ApplyMailboxOrRebase",
        };
        Err(StatusError {
            message: format!(
                "A repository in {} state is currently unsupported",
                unsupported_state
            ),
        })
    }
    fn write_short_staged<W: WriteColor + Write>(&self, writer: &mut W) {
        if self.index_diff.entries.is_empty() {
            return;
        }

        let mut color_spec = ColorSpec::new();
        let staged_color = Some(self.get_color(StatusColorSlot::Added));
        color_spec.set_fg(staged_color);
        for file in &self.index_diff.entries {
            writer.set_color(&color_spec).unwrap();
            writer
                .write_all(file.state.short_status_string().as_bytes())
                .unwrap();
            writer.write_all(b"  ").unwrap();
            writer.reset().unwrap();
            writer.write_all(file.name.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }
    fn write_short_unstaged<W: WriteColor + Write>(&self, writer: &mut W) {
        let unstaged_files: Vec<&StatusEntry> = self
            .work_tree_diff
            .entries
            .iter()
            .filter(|e| e.state != Status::New)
            .collect();
        if unstaged_files.is_empty() {
            return;
        }
        let mut color_spec = ColorSpec::new();
        let unstaged_color = Some(self.get_color(StatusColorSlot::Changed));
        color_spec.set_fg(unstaged_color);
        for file in unstaged_files {
            writer.set_color(&color_spec).unwrap();
            writer.write_all(b" ").unwrap();
            writer
                .write_all(file.state.short_status_string().as_bytes())
                .unwrap();
            writer.write_all(b" ").unwrap();
            writer.reset().unwrap();
            writer.write_all(file.name.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }

    fn write_short_untracked<W: WriteColor + Write>(&self, writer: &mut W) {
        let untracked_files: Vec<&StatusEntry> = self
            .work_tree_diff
            .entries
            .iter()
            .filter(|e| e.state == Status::New)
            .collect();
        if untracked_files.is_empty() {
            return;
        }
        let mut color_spec = ColorSpec::new();
        let untracked_color = Some(self.get_color(StatusColorSlot::Untracked));
        color_spec.set_fg(untracked_color);
        for file in untracked_files {
            writer.set_color(&color_spec).unwrap();
            writer.write_all(b"?? ").unwrap();
            writer.reset().unwrap();
            writer.write_all(file.name.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }

    fn get_oid(&self, reference_name: &str) -> Option<Oid> {
        match self.repo.find_reference(reference_name) {
            Err(_) => None,
            Ok(reference) => {
                let commit = reference.peel_to_commit().unwrap();
                Some(commit.id())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{BranchType, Commit, Repository, Signature, SubmoduleUpdateOptions, Time};
    use indoc::indoc;
    use std::fs;
    use temp_testdir::TempDir;
    use termcolor::Buffer;

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

    fn add_submodule(path: &Path, submodule_url: &str, submodule_path: &str) -> () {
        let repo = Repository::init(path).unwrap();
        let mut submodule = repo
            .submodule(submodule_url, Path::new(submodule_path), true)
            .unwrap();
        let mut submodule_options = SubmoduleUpdateOptions::new();
        submodule.clone(Some(&mut submodule_options)).unwrap();
        submodule.add_finalize().unwrap();

        let mut index = repo.index().unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
        let head = repo.head().unwrap().target().unwrap();
        let head = repo.find_commit(head).unwrap();
        repo.commit(
            Option::from("HEAD"),
            &signature,
            &signature,
            "Adding submodule",
            &tree,
            &[&head],
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
    fn test_branch_message_normal() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("a_file")]);

        repo.set_head("refs/heads/tip").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let mut writer = Buffer::no_color();
        status.write_branch_message(&mut writer).unwrap();
        assert_eq!(
            String::from_utf8(writer.into_inner()).unwrap(),
            "On branch tip\n"
        );
    }

    #[test]
    fn test_branch_message_different_branch() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("i_guess")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_branch_message(&mut writer).unwrap();
        assert_eq!(
            String::from_utf8(writer.into_inner()).unwrap(),
            "On branch half\n"
        );
    }

    #[test]
    fn test_branch_message_detached_parent() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let parent = head.parent(0).unwrap().id();
        repo.set_head_detached(parent).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_branch_message(&mut writer).unwrap();
        assert_eq!(
            String::from_utf8(writer.into_inner()).unwrap(),
            "Head detached at 17fe299\n"
        );
    }

    #[test]
    fn test_branch_message_detached_tip() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.set_head_detached(head.id()).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_branch_message(&mut writer).unwrap();
        assert_eq!(
            String::from_utf8(writer.into_inner()).unwrap(),
            "Head detached at 82578fa\n"
        );
    }

    #[test]
    fn test_remote_branch_difference_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);
        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        let expected = indoc! {"\
            Your branch is up to date with 'origin/tip'.

            "};
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_remote_branch_different_remote_name_same_spot() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);

        repo.set_head("refs/heads/half").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        let expected = indoc! {"\
            Your branch is up to date with 'origin/half'.

            "};
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        let expected = indoc! {"\
            Your branch is behind 'origin/tip' by 2 commits, and can be fast-forwarded.
              (use \"git pull\" to update your local branch)

            "};
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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
        let expected = indoc! {"\
            Your branch is ahead of 'origin/half' by 1 commit.
              (use \"git push\" to publish your local commits)

            "};
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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
        let expected = indoc! {"\
            Your branch is ahead of 'origin/half' by 3 commits.
              (use \"git push\" to publish your local commits)

            "};
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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
        let expected = indoc! {"\
            Your branch and 'origin/tip' have diverged,
            and have 3 and 2 different commits each, respectively.
              (use \"git pull\" to merge the remote branch into yours)

            "};
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_no_remote_branch() {
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &vec![Path::new("what")]);

        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("a_new_branch", &commit, false).unwrap();
        repo.set_head("refs/heads/a_new_branch").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), "");
    }

    #[test]
    fn test_no_modified_files() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_unstaged_message(&mut writer), false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), "");
    }

    #[test]
    fn test_one_modified_file() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[2], "what???");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   three

            "};

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_unstaged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   four
                    modified:   one/nested/a/bit.txt

            "};
        let mut writer = Buffer::no_color();
        assert_eq!(status.write_unstaged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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

        // Throw an untracked file in here as it should not show up in this message,
        // but it comes from the same source.
        write_to_file(&repo, Path::new("an_untracked_file"), "stuff");

        let status = RepoStatus::new(workdir).unwrap();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   one
                    deleted:    three
                    modified:   two

            "};

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_unstaged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn submodule_with_modified_files() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp = TempDir::default();
        let super_repo = temp.join("super_repo");
        let repo = test_repo(super_repo.to_str().unwrap(), &files);

        let sub_repo = temp.join("sub_repo");
        let sub_names = vec!["a_sub_file.md", "sure.c"];
        let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
        let sub_repo = test_repo(sub_repo.to_str().unwrap(), &sub_files);

        add_submodule(
            &super_repo.join("main"),
            sub_repo.workdir().unwrap().to_str().unwrap(),
            "sub_repo_dir",
        );

        let workdir = repo.workdir().unwrap();
        let modified_sub_repo_file = workdir.join("sub_repo_dir/sure.c");
        fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();

        let status = RepoStatus::new(&workdir).unwrap();

        let expected = indoc! {"\
            Changes not staged for commit:
              (use \"git add <file>...\" to update what will be committed)
              (use \"git restore <file>...\" to discard changes in working directory)
                    modified:   sub_repo_dir (modified content)

            "};

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_unstaged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_no_staged_files() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_staged_message(&mut writer), false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), "");
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

        let expected = indoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    new file:   a_new_file

            "};
        let mut writer = Buffer::no_color();
        assert_eq!(status.write_staged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
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

        let expected = indoc! {"\
            Changes to be committed:
              (use \"git restore --staged <file>...\" to unstage)
                    modified:   one
                    new file:   some_new_file
                    modified:   two

            "};
        let mut writer = Buffer::no_color();
        assert_eq!(status.write_staged_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_no_untracked_file() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_untracked_message(&mut writer), false);
    }

    #[test]
    fn test_untracked_file() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, Path::new("some_new_file"), "stuff");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = indoc! {"\
            Untracked files:
              (use \"git add <file>...\" to include in what will be committed)
                    some_new_file

            "};

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_untracked_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_untracked_file_and_directory() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, Path::new("b/path/to/a/file"), "stuff");
        write_to_file(&repo, Path::new("a_new_file"), "stuff");

        // A modified file to ensure that it doesn't show up in the untracked list, even though it
        // comes from the same source
        write_to_file(&repo, files[1], "stuff");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = indoc! {"\
            Untracked files:
              (use \"git add <file>...\" to include in what will be committed)
                    a_new_file
                    b/

            "};

        let mut writer = Buffer::no_color();
        assert_eq!(status.write_untracked_message(&mut writer), true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_no_change_epilog() {
        let expected = "nothing to commit, working tree clean\n".to_string();
        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, false, false, false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_unstaged_epilog() {
        let expected =
            "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n".to_string();
        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, false, true, false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_staged_epilog() {
        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, true, false, false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), "");
    }

    #[test]
    fn test_untracked_epilog() {
        let expected =
            "nothing added to commit but untracked files present (use \"git add\" to track)\n"
                .to_string();

        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, false, false, true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_unstaged_overrides_untracked_epilog() {
        let expected =
            "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n".to_string();
        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, false, true, true);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_staged_overrides_unstaged_epilog() {
        let mut writer = Buffer::no_color();
        RepoStatus::write_epilog(&mut writer, true, true, false);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), "");
    }

    #[test]
    fn test_default_untracked_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let color = status.get_color(StatusColorSlot::Untracked);

        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_overridden_untracked_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let mut config = repo.config().unwrap();
        config.set_str("color.status.untracked", "white").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let color = status.get_color(StatusColorSlot::Untracked);

        assert_eq!(color, Color::White);
    }

    #[test]
    fn test_default_changed_file_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[2], "what???");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let color = status.get_color(StatusColorSlot::Changed);

        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_overridden_changed_file_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let mut config = repo.config().unwrap();
        config.set_str("color.status.changed", "blue").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let color = status.get_color(StatusColorSlot::Changed);

        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_default_added_file_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let color = status.get_color(StatusColorSlot::Added);

        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_overridden_added_file_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let mut config = repo.config().unwrap();
        config.set_str("color.status.added", "yellow").unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let color = status.get_color(StatusColorSlot::Added);

        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_default_no_branch_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let parent = head.parent(0).unwrap().id();
        repo.set_head_detached(parent).unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let color = status.get_color(StatusColorSlot::NoBranch);

        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_overridden_no_branch_color() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let parent = head.parent(0).unwrap().id();
        repo.set_head_detached(parent).unwrap();
        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("color.status.nobranch", "white").unwrap();

        let color = status.get_color(StatusColorSlot::NoBranch);

        assert_eq!(color, Color::White);
    }

    #[test]
    fn short_message_file_added_to_index() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let new_file = Path::new("a_new_file");
        write_to_file(&repo, new_file, "stuff");
        stage_file(&repo, new_file);

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = indoc! {"\
            A  a_new_file
            "};
        let mut writer = Buffer::no_color();
        status.write_short_staged(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn short_message_two_modified_files() {
        let file_names = vec!["one/nested/a/bit.txt", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, files[0], "what???");
        write_to_file(&repo, files[3], "what???");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = " M four\n M one/nested/a/bit.txt\n";
        let mut writer = Buffer::no_color();
        status.write_short_unstaged(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn short_untracked_file() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        write_to_file(&repo, Path::new("some_new_file"), "stuff");

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();

        let expected = indoc! {"\
            ?? some_new_file
            "};
        let mut writer = Buffer::no_color();
        status.write_short_untracked(&mut writer);
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_upstream_branch_tip_gone() {
        let file_names = vec!["one", "two", "three", "four"];
        let files = file_names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = TempDir::default();
        let repo = test_repo(temp_dir.to_str().unwrap(), &files);

        let mut remote_branch = repo.find_branch("origin/tip", BranchType::Remote).unwrap();
        remote_branch.delete().unwrap();

        let status = RepoStatus::new(repo.workdir().unwrap()).unwrap();
        let mut writer = Buffer::no_color();
        status.write_remote_branch_difference_message(&mut writer);
        let expected = indoc! {"\
            Your branch is based on 'origin/tip', but the upstream is gone.
             (use \"git branch --unset-upstream\" to fixup)

            "};
        assert_eq!(String::from_utf8(writer.into_inner()).unwrap(), expected);
    }
}
