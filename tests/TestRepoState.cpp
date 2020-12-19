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
#include "Repo.hpp"
#include "Status.hpp"

TEST_CASE_METHOD(TempRepo, "Test repo state is empty") {
    auto status = Status(m_repo);
    std::stringstream stream;
    status.getRepoStateMessage(stream);

    REQUIRE("" == stream.str());

}

TEST_CASE_METHOD(TempRepo, "Test repo state has merge conflicts") {
    auto file_to_modify = m_dir / "sub_dir_1" / "sub_1_file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();
    addFile(file_to_modify);
    commit();

    branch("temp_branch");
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
    status.getRepoStateMessage(stream);

    REQUIRE("You have unmerged paths.\n"
            "  (fix conflicts and run \"git commit\")\n"
            "  (use \"git merge --abort\" to abort the merge)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test repo state is merging with no conflicts") {
    branch("temp_branch");
    merge("master");

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getRepoStateMessage(stream);

    REQUIRE("All conflicts fixed but you are still merging.\n"
            "  (use \"git commit\" to conclude merge)\n"
            "\n" == stream.str());
}
