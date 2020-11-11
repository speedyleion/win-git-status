//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#include <catch2/catch.hpp>
#include <fstream>
#include "TempRepo.hpp"
#include "Repo.hpp"

TEST_CASE_METHOD(TempRepo, "Test with a repo") {
    auto repo = Repo(m_dir.string());

    REQUIRE(std::string("On branch master\n"
            "Your branch is up to date with 'origin/master'.\n"
            "\n"
            "nothing to commit, working tree clean\n") == repo.status());

}

TEST_CASE_METHOD(TempRepo, "Test with an untracked file") {
    auto repo = Repo(m_dir.string());

    auto untracked = m_dir / "untracked.txt";
    auto file = std::ofstream(untracked);
    file << "This file is untracked\n";
    file.close();

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Untracked files:\n"
                        "  (use \"git add <file>...\" to include in what will be committed)\n"
                        "        untracked.txt\n"
                        "\n"
                        "nothing added to commit but untracked files present (use \"git add\" to track)\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with a modified file in working tree") {
    auto repo = Repo(m_dir.string());

    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes not staged for commit:\n"
                        "  (use \"git add <file>...\" to update what will be committed)\n"
                        "  (use \"git restore <file>...\" to discard changes in working directory)\n"
                        "        modified:   file_1.txt\n"
                        "\n"
                        "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with a renamed file in working tree") {
    auto repo = Repo(m_dir.string());

    auto old_name = m_dir / "file_3.txt";
    auto new_name = m_dir / "renamed.txt";
    std::filesystem::rename(old_name, new_name);

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes not staged for commit:\n"
                        "  (use \"git add <file>...\" to update what will be committed)\n"
                        "  (use \"git restore <file>...\" to discard changes in working directory)\n"
                        "        deleted:    file_3.txt\n"
                        "\n"
                        "Untracked files:\n"
                        "  (use \"git add <file>...\" to include in what will be committed)\n"
                        "        renamed.txt\n"
                        "\n"
                        "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n") == repo.status());
}
