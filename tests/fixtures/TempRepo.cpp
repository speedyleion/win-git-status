//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "TempRepo.hpp"

#include <catch2/catch.hpp>
#include <git2/branch.h>
#include <git2/index.h>
#include <git2/clone.h>
#include <git2/submodule.h>
#include <git2/revparse.h>
#include <git2/commit.h>
#include <iostream>
#include "TempDirectory.hpp"
#include "TempRepo.hpp"
#include "RepoBuilder.hpp"

int submodule_update(git_submodule *sm, const char *name, void *payload){
    git_submodule_update_options options = GIT_SUBMODULE_UPDATE_OPTIONS_INIT;
    return git_submodule_update(sm, 1, &options);
}

TempRepo::TempRepo() {
    auto name = Catch::getResultCapture().getCurrentTestName();
    std::transform(name.begin(), name.end(), name.begin(), ::tolower);
    std::replace(name.begin(), name.end(), ' ', '_');
    std::replace(name.begin(), name.end(), ',', '_');
    std::replace(name.begin(), name.end(), '.', '_');
    m_dir = TempDirectory::TempDir(name);
    auto origin = RepoBuilder::getOriginRepo();
    git_clone(&m_repo, origin.c_str(), m_dir.string().c_str(), NULL);
    git_submodule_foreach(m_repo, submodule_update, NULL);
}

TempRepo::~TempRepo() {
    git_repository_free(m_repo);
}

void TempRepo::addFile(const std::string &filename, const std::string &submodule_path) {
    auto repo = m_repo;
    if(!submodule_path.empty()) {
        git_submodule * sub_module;
        git_submodule_lookup(&sub_module, m_repo, submodule_path.c_str());
        git_submodule_open(&repo, sub_module);
    }

    git_index * index;
    git_repository_index(&index, repo);
    git_index_add_bypath(index, filename.c_str());
    git_index_write(index);
    git_index_free(index);

    if(!submodule_path.empty()) {
        git_repository_free(repo);
    }
}

void TempRepo::removeFile(const std::string &filename) {
    git_index * index;
    git_repository_index(&index, m_repo);
    git_index_remove_bypath(index, filename.c_str());
    git_index_write(index);
    git_index_free(index);

}

void TempRepo::commit(const std::string &submodule_path) {
    auto repo = m_repo;
    if(!submodule_path.empty()) {
        git_submodule * sub_module;
        git_submodule_lookup(&sub_module, m_repo, submodule_path.c_str());
        git_submodule_open(&repo, sub_module);
    }
    git_signature signature = {"Tucan", "somewhere@foo.bar", 1000};

    git_index * index;
    git_repository_index(&index, repo);
    git_oid tree_oid;
    git_index_write_tree(&tree_oid, index);
    git_tree * tree;
    git_tree_lookup(&tree, repo, &tree_oid);

    git_object * parent = NULL;
    git_reference * ref = NULL;
    git_revparse_ext(&parent, &ref, repo, "HEAD");

    git_commit_create_v(&tree_oid, repo, "HEAD", &signature, &signature, NULL, "This is a test", tree,
                        parent ? 1 : 0, parent);

    git_object_free(parent);
    git_reference_free(ref);
    git_index_free(index);
    git_tree_free(tree);
    if(!submodule_path.empty()) {
        git_repository_free(repo);
    }
}

void TempRepo::branch(const std::string &branch_name) {
    git_commit *commit = NULL;
    git_reference * head = NULL;
    git_repository_head(&head, m_repo);
    auto oid = git_reference_target(head);
    git_commit_lookup(&commit, m_repo, oid);
    git_reference_free(head);

    git_reference * reference = NULL;
    git_branch_create(&reference, m_repo, branch_name.c_str(), commit, 0);
    git_repository_set_head(m_repo, git_reference_name(reference));
    git_commit_free(commit);
    git_reference_free(reference);

}
