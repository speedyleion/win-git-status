/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
extern crate win_git_status;
use git2::Repository;
use temp_testdir::TempDir;
use win_git_status::Index;
mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let temp_path_str = temp.to_str().unwrap();
    common::test_repo(temp_path_str);
    let index_file = temp.join(".git/index");
    let index = Index::new(&index_file).unwrap();
    assert_eq!(
        index.entries.len(),
        1
    );
}
