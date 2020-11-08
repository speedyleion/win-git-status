//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "RepoBuilder.hpp"

#include "RepoBuilder.hpp"

#include "git2/repository.h"
#include "git2/global.h"
#include "git2/index.h"
#include "git2/commit.h"
#include "git2/tree.h"
#include "git2/refs.h"
#include "git2/revparse.h"
#include "git2/submodule.h"

std::string RepoBuilder::m_origin;

RepoBuilder::RepoBuilder(const std::string &path) {
    git_libgit2_init();

    m_repo = NULL;
    git_repository_init(&m_repo, path.c_str(), 0);
}
RepoBuilder::~RepoBuilder(){
    git_repository_free(m_repo);
}

void RepoBuilder::addSubmodule(const std::string &url, const std::string &path) {
    git_submodule *submodule;
    git_submodule_add_setup(&submodule, m_repo, url.c_str(), path.c_str(), 1);
    git_submodule_update_options options = GIT_SUBMODULE_UPDATE_OPTIONS_INIT;
    git_submodule_clone(NULL, submodule, &options);
    git_submodule_add_finalize(submodule);
    git_submodule_free(submodule);
}

void RepoBuilder::addFile(std::string filename) {
    git_index * index;
    git_repository_index(&index, m_repo);
    git_index_add_bypath(index, filename.c_str());
    git_index_write(index);
    git_index_free(index);
}

void RepoBuilder::commit(std::string message) {
    git_signature signature = {"Tucan", "somewhere@foo.bar", m_time++};

    git_index * index;
    git_repository_index(&index, m_repo);
    git_oid tree_oid;
    git_index_write_tree(&tree_oid, index);
    git_tree * tree;
    git_tree_lookup(&tree, m_repo, &tree_oid);

    git_object * parent = NULL;
    git_reference * ref = NULL;
    git_revparse_ext(&parent, &ref, m_repo, "HEAD");

    git_commit_create_v(&tree_oid, m_repo, "HEAD", &signature, &signature, NULL, message.c_str(), tree,
                        parent ? 1 : 0, parent);

    git_object_free(parent);
    git_reference_free(ref);
    git_index_free(index);
    git_tree_free(tree);
}

void RepoBuilder::setOriginRepo(const std::string &url) {
    m_origin = url;
}

const std::string RepoBuilder::getOriginRepo() {
    return m_origin;
}
