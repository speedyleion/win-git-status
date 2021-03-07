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
use std::collections::HashMap;

mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str, vec![Path::new("some_file.txt")]);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries.get("").unwrap().len(), 1);
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

    assert_eq!(index.entries.len(), 1);
    let dir_list = index.entries.get("").unwrap();
    let index_names: Vec<&String> = dir_list.iter().map(|e| &e.name).collect();
    names.sort();
    assert_eq!(index_names, names);
}

#[test]
fn index_has_nested_entries_in_order() {
    let mut names = vec!["dir_3/file_2.txt", "dir_2/file_1.txt", "absolute.md"];
    let files = names.iter().map(|n| Path::new(n)).collect();
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str, files);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();

    // let mut index_names = HashMap::new();
    // index.entries.into_iter().map(|(k, v)| index_names.insert(k, v.iter().map(|e| &e.name))).collect();
    let mut file_map = HashMap::new();
    names.sort();
    for file in names.iter().map(|n| Path::new(n)) {
        let mut entry = file_map.entry(file.parent().unwrap().to_str().unwrap()).or_insert(vec![]);
        entry.push(file.file_name().unwrap().to_str().unwrap());
    }
    assert_eq!(index.entries.len(), file_map.len());
    for (key, value) in index.entries.into_iter() {
        let index_names: Vec<&String> = value.iter().map(|e| &e.name).collect();
        assert_eq!(&index_names, file_map.get(key.as_str()).unwrap());
    }
}
