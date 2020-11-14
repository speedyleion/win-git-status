//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <TempRepo.hpp>
#include <Repo.hpp>

TEST_CASE_METHOD(TempRepo, "Create repo from on disk repo") {
    auto repo = Repo(m_dir.string());

    auto expected_dir = m_dir / ".git/";
    auto actual_dir = std::filesystem::path(repo.toString());
    REQUIRE(actual_dir.lexically_normal() == expected_dir.lexically_normal());
}

TEST_CASE_METHOD(TempRepo, "Attempt to create repo from non existent repo.") {
    REQUIRE_THROWS_AS(Repo("/something/that/should/not/exist"), RepoException);
}

TEST_CASE_METHOD(TempRepo, "Create a repo from a sub directory of the actual .git folder.") {
    auto sub_dir = m_dir / "sub_dir_1";
    auto repo = Repo(sub_dir.string());

    auto expected_dir = m_dir / ".git/";
    auto actual_dir = std::filesystem::path(repo.toString());
    REQUIRE(actual_dir.lexically_normal() == expected_dir.lexically_normal());
}
