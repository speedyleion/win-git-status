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

    m_repo = NULL;
    git_repository_open(&m_repo, path.c_str());
}

Repo::~Repo(){
    git_repository_free(m_repo);
}

std::string Repo::status() {
    Status status = Status(m_repo);
    std::stringstream stream;
    status.toStream(stream);
    return stream.str();
}


