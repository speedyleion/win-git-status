/*
 *          Copyright Nick G. 2020.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
use git2::{Commit, Repository, Signature, SubmoduleUpdateOptions, Time};
use std::fs;
use std::path::Path;

pub fn test_repo(path: &Path, files: Vec<&Path>) -> Repository {
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
    {
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
    repo
}

pub fn add_submodule(path: &Path, submodule_url: &str, submodule_path: &str) -> () {
    let repo = Repository::init(path).unwrap();
    let mut submodule = repo
        .submodule(submodule_url, Path::new(submodule_path), true)
        .unwrap();
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
    )
    .unwrap();
}

pub fn write_to_file(repo: &Repository, file: &Path, contents: &str) {
    let root = repo.workdir().unwrap();
    let full_path = root.join(file);

    // Done this way to support nested files
    fs::create_dir_all(full_path.parent().unwrap()).unwrap();
    fs::write(&full_path, contents).unwrap();
}

pub fn stage_file(repo: &Repository, file: &Path) {
    let mut index = repo.index().unwrap();
    index.add_path(&file).unwrap();
    index.write().unwrap();
}

pub fn commit_file(repo: &Repository, file: &Path) {
    write_to_file(repo, file, file.to_str().unwrap());
    stage_file(repo, file);
    let mut index = repo.index().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let signature = Signature::new("Tucan", "me@me.com", &Time::new(20, 0)).unwrap();
    let head = repo.head();
    let _parents = match head {
        Err(_) => vec![],
        _ => vec![head.unwrap().peel_to_commit().unwrap()],
    };
    let parents: Vec<&Commit> = _parents.iter().map(|n| n).collect();
    let message = "Commiting file: ".to_string() + file.to_str().unwrap();
    repo.commit(
        Option::from("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &parents[..],
    )
    .unwrap();
}
