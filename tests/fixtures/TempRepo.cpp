//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "TempRepo.hpp"

#include <catch2/catch.hpp>
#include "TempDirectory.hpp"
#include "TempRepo.hpp"
#include "RepoBuilder.hpp"
#include "git2/types.h"
#include "git2/clone.h"

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
