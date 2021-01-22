/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
use git2::{Repository, Signature, Time};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn test_repo(path: &str) -> () {
    let repo = Repository::init(path).unwrap();
    let mut index = repo.index().unwrap();
    let root = repo.path().parent().unwrap();
    let mut file = File::create(&root.join("foo.txt")).unwrap();
    file.write(b"Stuff").unwrap();
    index.add_path(Path::new("foo.txt")).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
    repo.commit(
        Option::from("HEAD"),
        &signature,
        &signature,
        "A message",
        &tree,
        // No parents yet this is the first commit
        &[],
    )
    .unwrap();
}
