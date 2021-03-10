/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

extern crate win_git_status;
use std::path::Path;
use temp_testdir::TempDir;
use win_git_status::{Index, WorkTree};
use win_git_status::worktree::{Status, WorkTreeEntry};
use std::fs;

mod common;

#[test]
fn worktree_diff_with_submodule() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(super_repo.to_str().unwrap(), files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(sub_repo.to_str().unwrap(), sub_files);

    common::add_submodule(super_repo.to_str().unwrap(), sub_repo.to_str().unwrap(), "sub_repo_dir");

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    assert_eq!(value.entries, vec![]);
}

#[test]
fn worktree_diff_with_dirty_submodule() {
    let temp = TempDir::default().permanent();
    let super_repo = temp.join("super_repo");
    let names = vec!["a_file.txt", "another.md", "what.log"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(super_repo.to_str().unwrap(), files);

    let sub_repo = temp.join("sub_repo");
    let sub_names = vec!["a_sub_file.md", "sure.c"];
    let sub_files = sub_names.iter().map(|n| Path::new(n)).collect();
    common::test_repo(sub_repo.to_str().unwrap(), sub_files);

    common::add_submodule(super_repo.to_str().unwrap(), sub_repo.to_str().unwrap(), "sub_repo_dir");

    let new_sub_repo_file = sub_repo.join("new_file.txt");
    fs::write(&new_sub_repo_file, "stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![WorkTreeEntry {
        name: "sub_repo/".to_string(),
        state: Status::MODIFIED,
    }];

    assert_eq!(value.entries, entries);
}
