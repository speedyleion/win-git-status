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
                        "        \u001b[31muntracked.txt\u001b[0m\n"
                        "\n"
                        "nothing added to commit but untracked files present (use \"git add\" to track)\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with new file added to index") {
    auto repo = Repo(m_dir.string());

    std::string filename = "untracked.txt";
    auto untracked = m_dir / filename;
    auto file = std::ofstream(untracked);
    file << "This file is untracked\n";
    file.close();

    addFile(filename);

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes to be committed:\n"
                        "  (use \"git restore --staged <file>...\" to unstage)\n"
                        "        \u001b[32mnew file:   untracked.txt\u001b[0m\n"
                        "\n") == repo.status());
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
                        "        \u001b[31mmodified:   file_1.txt\u001b[0m\n"
                        "\n"
                        "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with a modified file added to index") {
    auto repo = Repo(m_dir.string());

    std::string filename = "file_1.txt";
    auto file_to_modify = m_dir / filename;
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();

    addFile(filename);

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes to be committed:\n"
                        "  (use \"git restore --staged <file>...\" to unstage)\n"
                        "        \u001b[32mmodified:   file_1.txt\u001b[0m\n"
                        "\n") == repo.status());
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
                        "        \u001b[31mdeleted:    file_3.txt\u001b[0m\n"
                        "\n"
                        "Untracked files:\n"
                        "  (use \"git add <file>...\" to include in what will be committed)\n"
                        "        \u001b[31mrenamed.txt\u001b[0m\n"
                        "\n"
                        "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with a renamed file in index") {
    auto repo = Repo(m_dir.string());

    std::string old_name = "file_3.txt";
    std::string new_name = "renamed.txt";
    auto old_file = m_dir / old_name;
    auto new_file = m_dir / new_name;
    std::filesystem::rename(old_file, new_file);

    removeFile(old_name);
    addFile(new_name);

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes to be committed:\n"
                        "  (use \"git restore --staged <file>...\" to unstage)\n"
                        "        \u001b[32mrenamed:    file_3.txt -> renamed.txt\u001b[0m\n"
                        "\n") == repo.status());
}

TEST_CASE_METHOD(TempRepo, "Test with a deleted file in index") {
    auto repo = Repo(m_dir.string());

    removeFile("file_2.txt");

    REQUIRE(std::string("On branch master\n"
                        "Your branch is up to date with 'origin/master'.\n"
                        "\n"
                        "Changes to be committed:\n"
                        "  (use \"git restore --staged <file>...\" to unstage)\n"
                        "        \u001b[32mdeleted:    file_2.txt\u001b[0m\n"
                        "\n"
                        "Untracked files:\n"
                        "  (use \"git add <file>...\" to include in what will be committed)\n"
                        "        \u001b[31mfile_2.txt\u001b[0m\n"
                        "\n") == repo.status());
}

