//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "Repo.hpp"

#include <sstream>
#include <git2/global.h>
#include <git2/repository.h>
#include "Repo.hpp"
#include "Status.hpp"

Repo::Repo(const std::string &path) {
    git_libgit2_init();

    git_buf repo_path_buffer={0};
    git_repository_discover(&repo_path_buffer, path.c_str(), 0, NULL);
    m_repo = NULL;
    if(git_repository_open(&m_repo, repo_path_buffer.ptr) != 0 ){
        throw RepoException("fatal: not a git repository (or any of the parent directories): .git");
    }
}

Repo::~Repo(){
    git_repository_free(m_repo);
}

std::string Repo::status() {
    Status status = Status(m_repo);
    std::stringstream stream;
    status.toStream(stream, Colorize::COLORIZE);
    return stream.str();
}

std::string Repo::toString() {
    return git_repository_commondir(m_repo);
}


