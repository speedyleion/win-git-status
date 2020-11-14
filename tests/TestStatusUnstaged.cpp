//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <Status.hpp>
#include <sstream>
#include <fstream>
#include <git2/repository.h>
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

TEST_CASE_METHOD(TempRepo, "Test nested file modified") {
    auto file_to_modify = m_dir / "sub_dir_1" / "sub_1_file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "        modified:   sub_dir_1/sub_1_file_1.txt\n"
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

TEST_CASE_METHOD(TempRepo, "Test sub repo with modified content") {
    auto modified = m_dir / "sub_repo_1" / "file_1.txt";
    auto file = std::ofstream(modified);
    file << "This file is modified\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (modified content)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test sub repo with staged content") {
    std::string sub_repo_dir = "sub_repo_1";
    std::string filename = "file_1.txt";
    auto modified = m_dir / sub_repo_dir /  filename;
    auto file = std::ofstream(modified);
    file << "This file is modified\n";
    file.close();

    addFile(filename, sub_repo_dir);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (modified content)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test sub repo with staged and untracked content") {
    std::string sub_repo_dir = "sub_repo_1";
    std::string filename = "file_1.txt";
    auto modified = m_dir / sub_repo_dir / filename;
    auto file = std::ofstream(modified);
    file << "This file is modified\n";
    file.close();

    addFile(filename, sub_repo_dir);

    auto untracked = m_dir / sub_repo_dir / "foo.txt";
    file = std::ofstream(untracked);
    file << "This file is untracked\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (modified content, untracked content)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test sub repo with new commits") {
    std::string sub_repo_dir = "sub_repo_1";
    std::string filename = "file_1.txt";
    auto modified = m_dir / sub_repo_dir / filename;
    auto file = std::ofstream(modified);
    file << "This file is modified\n";
    file.close();

    addFile(filename, sub_repo_dir);
    commit(sub_repo_dir);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (new commits)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test sub repo with new commits, modified content, and untracked content") {
    std::string sub_repo_dir = "sub_repo_1";
    std::string filename = "file_1.txt";
    auto modified = m_dir / sub_repo_dir / filename;
    auto file = std::ofstream(modified);
    file << "This file is modified\n";
    file.close();

    addFile(filename, sub_repo_dir);
    commit(sub_repo_dir);

    file = std::ofstream(modified);
    file << "This file is further modified\n";
    file.close();

    auto untracked = m_dir / sub_repo_dir / "foo.txt";
    file = std::ofstream(untracked);
    file << "This file is untracked\n";
    file.close();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "  (commit or discard the untracked or modified content in submodules)\n"
            "        modified:   sub_repo_1 (new commits, modified content, untracked content)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test path relative when called from sub directory", "[.not_implemented]") {
    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();

    auto sub_folder = m_dir / "sub_dir_2";
    git_repository *repo;
    git_repository_open(&repo, sub_folder.string().c_str());
    auto status = Status(repo);
    std::stringstream stream;
    status.getTrackedMessage(stream);
    git_repository_free(repo);

    REQUIRE("Changes not staged for commit:\n"
            "  (use \"git add <file>...\" to update what will be committed)\n"
            "  (use \"git restore <file>...\" to discard changes in working directory)\n"
            "        modified:   ../file_1.txt\n"
            "\n" == stream.str());
}
