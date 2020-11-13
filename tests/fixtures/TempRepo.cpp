//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "TempRepo.hpp"

#include <catch2/catch.hpp>
#include <git2/index.h>
#include <git2/clone.h>
#include "TempDirectory.hpp"
#include "TempRepo.hpp"
#include "RepoBuilder.hpp"

TempRepo::TempRepo() {
    auto name = Catch::getResultCapture().getCurrentTestName();
    std::transform(name.begin(), name.end(), name.begin(), ::tolower);
    std::replace(name.begin(), name.end(), ' ', '_');
    m_dir = TempDirectory::TempDir(name);
    auto origin = RepoBuilder::getOriginRepo();
    git_clone(&m_repo, origin.c_str(), m_dir.string().c_str(), NULL);
}

TempRepo::~TempRepo() {
    git_repository_free(m_repo);
}

void TempRepo::addFile(const std::string &filename) {
    git_index * index;
    git_repository_index(&index, m_repo);
    git_index_add_bypath(index, filename.c_str());
    git_index_write(index);
    git_index_free(index);
}

void TempRepo::removeFile(const std::string &filename) {
    git_index * index;
    git_repository_index(&index, m_repo);
    git_index_remove_bypath(index, filename.c_str());
    git_index_write(index);
    git_index_free(index);

}
