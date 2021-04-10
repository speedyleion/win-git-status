/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

extern crate win_git_status;
use std::fs;
use std::path::Path;
use temp_testdir::TempDir;
use win_git_status::status::{Status, StatusEntry};
use win_git_status::{Index, WorkTree};
use git2::Repository;

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

    common::add_submodule(
        &super_repo,
        sub_repo.to_str().unwrap(),
        "sub_repo_dir",
    );

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

    common::add_submodule(
        &super_repo,
        sub_repo.to_str().unwrap(),
        "sub_repo_dir",
    );

    let new_sub_repo_file = super_repo.join("sub_repo_dir/new_file.txt");
    fs::write(&new_sub_repo_file, "stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified,
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

    common::add_submodule(
        &super_repo,
        sub_repo.to_str().unwrap(),
        "sub_repo_dir",
    );

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified,
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

    common::add_submodule(
        &super_repo,
        sub_repo_name.to_str().unwrap(),
        "sub_repo_dir",
    );

    let modified_sub_repo_file = super_repo.join("sub_repo_dir/sure.c");
    fs::write(&modified_sub_repo_file, "some modified stuff").unwrap();
    let sub_repo = Repository::open(super_repo.join("sub_repo_dir")).unwrap();
    let mut repo_index = sub_repo.index().unwrap();
    repo_index.add_path(Path::new("sure.c")).unwrap();
    repo_index.write().unwrap();

    let index_file = super_repo.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let value = WorkTree::diff_against_index(&super_repo, index).unwrap();
    let entries = vec![StatusEntry {
        name: "sub_repo_dir".to_string(),
        state: Status::Modified,
    }];

    assert_eq!(value.entries, entries);
}
//  Behaviour needed for submodules
//
//  modified:   <red>sub_repo_dir</red> (untracked content)
//      Has an untracked file.
//
//  modified:   <red>sub_repo_dir</red> (new commits)
//      has a different commit, can be newer or older doesn't matter
//
//  modified:   <red>sub_repo_dir</red> (modified content)
//      has a changed file in the working tree, or has a staged file
//
//  modified:   <red>sub_repo_dir</red> (new commits, untracked content)
//      has a different commit and an untracked file
//
//  modified:   <red>sub_repo_dir</red> (new commits, modified content)
//      has a different commit and a modified file
//
//  modified:   <red>sub_repo_dir</red> (new commits, modified content, untracked content)
//      has a different commit, a modified file, and an untracked file



