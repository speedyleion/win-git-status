//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <sstream>
#include <git2/repository.h>
#include <git2/revparse.h>
#include <git2/reset.h>
#include <fstream>
#include "TempRepo.hpp"
#include "Status.hpp"

TEST_CASE_METHOD(TempRepo, "Test no merge from getUnmergedMessage") {
    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_merged_message = status.getUnmergedMessage(stream);

    REQUIRE("" == stream.str());

    REQUIRE(has_merged_message == false);
}

TEST_CASE_METHOD(TempRepo, "Test file_1 merge conflict") {

    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();
    addFile(file_to_modify);
    commit();

    branch("merge_branch");
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~1");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    file = std::ofstream(file_to_modify);
    file << "Something else happened here\n";
    file.close();
    addFile(file_to_modify);
    commit();

    merge("master");

    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_merged_message = status.getUnmergedMessage(stream);

    REQUIRE("Unmerged paths:\n"
            "  (use \"git add <file>...\" to mark resolution)\n"
            "        both modified:   file_1.txt\n"
            "\n" == stream.str());

    REQUIRE(has_merged_message == true);
}

TEST_CASE_METHOD(TempRepo, "Test sub_dir_2 sub_2_file_3 merge conflict") {

    auto file_to_modify = m_dir / "sub_dir_2" / "sub_2_file_3.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();
    addFile(file_to_modify);
    commit();

    branch("merge_branch");
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~1");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    file = std::ofstream(file_to_modify);
    file << "Something else happened here\n";
    file.close();
    addFile(file_to_modify);
    commit();

    merge("master");

    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_merged_message = status.getUnmergedMessage(stream);

    REQUIRE("Unmerged paths:\n"
            "  (use \"git add <file>...\" to mark resolution)\n"
            "        both modified:   sub_dir_2/sub_2_file_3.txt\n"
            "\n" == stream.str());

    REQUIRE(has_merged_message == true);
}

TEST_CASE_METHOD(TempRepo, "Test file_1 merge conflict with color") {

    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();
    addFile(file_to_modify);
    commit();

    branch("merge_branch");
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~1");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    file = std::ofstream(file_to_modify);
    file << "Something else happened here\n";
    file.close();
    addFile(file_to_modify);
    commit();

    merge("master");

    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_merged_message = status.getUnmergedMessage(stream, Colorize::COLORIZE);

    REQUIRE("Unmerged paths:\n"
            "  (use \"git add <file>...\" to mark resolution)\n"
            "        \x1B[31mboth modified:   file_1.txt\x1B[0m\n"
            "\n" == stream.str());

    REQUIRE(has_merged_message == true);
}