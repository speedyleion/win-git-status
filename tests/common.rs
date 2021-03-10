/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
use git2::{Repository, Signature, Time, SubmoduleUpdateOptions};
use std::fs;
use std::path::Path;

pub fn test_repo(path: &str, files: Vec<&Path>) -> () {
    let repo = Repository::init(path).unwrap();
    let mut index = repo.index().unwrap();
    let root = repo.path().parent().unwrap();
    for file in files {
        let full_path = root.join(file);

        // Done this way to support nested files
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, file.to_str().unwrap()).unwrap();
        index.add_path(&file).unwrap();
    }
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

pub fn add_submodule(path: &str, submodule_url: &str, submodule_path: &str) -> () {
    let repo = Repository::init(path).unwrap();
    let mut submodule = repo.submodule(submodule_url, Path::new(submodule_path), true).unwrap();
    let mut submodule_options = SubmoduleUpdateOptions::new();
    submodule.clone(Some(&mut submodule_options)).unwrap();
    submodule.add_finalize().unwrap();

    let mut index = repo.index().unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    let head = repo.find_commit(head).unwrap();
    repo.commit(
        Option::from("HEAD"),
        &signature,
        &signature,
        "Adding submodule",
        &tree,
        &[&head],
    ).unwrap();
}