//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#include <catch2/catch.hpp>
#include "TempRepo.hpp"
#include "Repo.hpp"

TEST_CASE_METHOD(TempRepo, "Test with a repo") {
    auto repo = Repo(m_dir.string());

    REQUIRE(std::string("On branch master\n"
            "Your branch is up to date with 'origin/master'.\n"
            "\n"
            "nothing to commit, working tree clean\n") == repo.status());

}