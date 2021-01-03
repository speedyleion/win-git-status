/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
extern crate win_git_status;
use temp_testdir::TempDir;
use win_git_status::Index;
mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let repo = common::test_repo(temp.to_str().unwrap());
    let mut index = repo.index().unwrap();
    assert_eq!(index.write_tree().unwrap().as_bytes(), Index::new(&temp).oid());
}