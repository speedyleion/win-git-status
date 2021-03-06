/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
extern crate win_git_status;
use std::path::Path;
use temp_testdir::TempDir;
use win_git_status::Index;

mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str, vec![Path::new("some_file.txt")]);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();
    assert_eq!(index.entries.len(), 1);
}

#[test]
fn index_has_three_entries_in_order() {
    let mut names = vec!["one.baz", "two.txt", "three.md"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str, files);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let index_names: Vec<&String> = index.entries.iter().map(|e| &e.name).collect();
    names.sort();
    assert_eq!(index_names, names);
}

#[test]
fn index_has_nested_entries_in_oder() {
    let mut names = vec!["dir_3/file_2.txt", "dir_2/file_1.txt", "absolut.md"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str, files);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    let index_names: Vec<&String> = index.entries.iter().map(|e| &e.name).collect();
    names.sort();
    assert_eq!(index_names, names);
}
