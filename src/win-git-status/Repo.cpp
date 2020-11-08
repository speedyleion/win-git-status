//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "Repo.hpp"

#include <git2/global.h>
#include <git2/repository.h>
#include "Repo.hpp"

Repo::Repo(const std::string &path) {
    git_libgit2_init();

    m_repo = NULL;
    git_repository_open(&m_repo, path.c_str());
}

Repo::~Repo(){
    git_repository_free(m_repo);
}

std::string Repo::status() {
    return std::string("On branch master\n"
                       "Your branch is up to date with 'origin/master'.\n"
                       "\n"
                       "nothing to commit, working tree clean\n");

}
