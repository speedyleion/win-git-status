/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

extern crate win_git_status;
use git2::Repository;
use std::fs;
use std::path::Path;
use temp_testdir::TempDir;
use win_git_status::status::{Status, StatusEntry};
use win_git_status::{Index, WorkTree};

mod common;

#[test]
fn worktree_diff_with_submodule() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo, sub_files);

    common::add_submodule(&super_repo, sub_repo.to_str().unwrap(), "sub_repo_dir");

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    assert_eq!(value.entries, vec![]);
}

#[test]
fn submodule_with_new_file() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo, sub_files);

    common::add_submodule(&super_repo, sub_repo.to_str().unwrap(), "sub_repo_dir");

    let new_sub_repo_file = super_repo.join("sub_repo_dir/new_file.txt");
    fs::write(&new_sub_repo_file, "stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from("untracked content"))),
    }];

    assert_eq!(value.entries, entries);
}

#[test]
fn submodule_with_modified_files() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo, sub_files);

    common::add_submodule(&super_repo, sub_repo.to_str().unwrap(), "sub_repo_dir");

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from("modified content"))),
    }];

    assert_eq!(value.entries, entries);
}

#[test]
fn submodule_with_staged_files() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo_name = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo_name, sub_files);

    common::add_submodule(&super_repo, sub_repo_name.to_str().unwrap(), "sub_repo_dir");

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();
    let sub_repo = Repository::open(super_repo.join("sub_repo_dir")).unwrap();
    common::stage_file(&sub_repo, Path::new("sure.c"));

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from("modified content"))),
    }];

    assert_eq!(value.entries, entries);
}

#[test]
fn submodule_with_new_commits() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo_name = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo_name, sub_files);

    common::add_submodule(&super_repo, sub_repo_name.to_str().unwrap(), "sub_repo_dir");

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();
    let sub_repo = Repository::open(super_repo.join("sub_repo_dir")).unwrap();
    let local_file_path = Path::new("sure.c");
    common::stage_file(&sub_repo, &local_file_path);
    common::commit_file(&sub_repo, &local_file_path);

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from("new commits"))),
    }];

    assert_eq!(value.entries, entries);
}

#[test]
fn worktree_diff_with_submodule_removed() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo, sub_files);

    common::add_submodule(&super_repo, sub_repo.to_str().unwrap(), "sub_repo_dir");

    fs::remove_dir_all(super_repo.join("sub_repo_dir")).unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    assert_eq!(value.entries, vec![]);
}

#[test]
fn submodule_with_new_file_and_modified_file() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo, sub_files);

    common::add_submodule(&super_repo, sub_repo.to_str().unwrap(), "sub_repo_dir");

    let new_sub_repo_file = super_repo.join("sub_repo_dir/new_file.txt");
    fs::write(&new_sub_repo_file, "stuff").unwrap();

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from("modified content, untracked content"))),
    }];

    assert_eq!(value.entries, entries);
}

#[test]
fn submodule_with_new_commits_staged_files_and_untracked_file() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&super_repo, files);

    let sub_repo_name = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(&sub_repo_name, sub_files);

    common::add_submodule(&super_repo, sub_repo_name.to_str().unwrap(), "sub_repo_dir");

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();
    let sub_repo = Repository::open(super_repo.join("sub_repo_dir")).unwrap();
    let local_file_path = Path::new("sure.c");
    common::stage_file(&sub_repo, &local_file_path);
    common::commit_file(&sub_repo, &local_file_path);

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/a_sub_file.md");
    fs::write(&modified_sub_repo_file, "whatever we want").unwrap();
    let sub_repo = Repository::open(super_repo.join("sub_repo_dir")).unwrap();
    common::stage_file(&sub_repo, Path::new("a_sub_file.md"));

    let new_sub_repo_file = super_repo.join("sub_repo_dir/new_file.txt");
    fs::write(&new_sub_repo_file, "stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified(Some(String::from(
            "new commits, modified content, untracked content",
        ))),
    }];

    assert_eq!(value.entries, entries);
}
