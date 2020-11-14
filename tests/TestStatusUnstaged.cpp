//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <Status.hpp>
#include <sstream>
#include <fstream>
#include "TempRepo.hpp"

TEST_CASE_METHOD(TempRepo, "Test no changes") {
    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_message = status.getTrackedMessage(stream);

    REQUIRE("" == stream.str());
    REQUIRE(has_message == false);
}

TEST_CASE_METHOD(TempRepo, "Test file deleted") {
    std::filesystem::remove(m_dir / "file_3.txt");
    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_message = status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "        deleted:    file_3.txt\n"
            "\n" == stream.str());

    REQUIRE(has_message == true);

}

TEST_CASE_METHOD(TempRepo, "Test file modified") {
    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "        modified:   file_1.txt\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test sub repo with untracked content") {
    auto untracked = m_dir / "sub_repo_1" / "foo.txt";
    auto file = std::ofstream(untracked);
    file << "This file is untracked\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (untracked content)\n"
            "\n" == stream.str());
}
/*
 * The way the sumbodules appear in the git command line, it looks like they get decorated with the status of their submodules as well.
 *
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
  (commit or discard the untracked or modified content in submodules)
        modified:   file_1.txt
        modified:   sub_repo_1 (new commits, modified content, untracked content)
        */

