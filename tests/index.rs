/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
extern crate win_git_status;
use temp_testdir::TempDir;
use win_git_status::Index;
use git2::Repository;
mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str);
    let repo = Repository::open(temp_path_str).unwrap();
    let mut index = repo.index().unwrap();
    assert_eq!(index.write_tree().unwrap().as_bytes(), Index::new(&temp).oid());
}