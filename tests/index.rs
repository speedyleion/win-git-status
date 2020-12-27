/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
use temp_testdir::TempDir;
mod common;

#[test]
fn index_has_one_entry() {
    let temp = TempDir::default().permanent();
    let oid = common::test_repo(temp.to_str().unwrap()).unwrap();
    assert_eq!(oid.as_bytes(), &[3,4]);
}