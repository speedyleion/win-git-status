//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <sstream>
#include <git2/repository.h>
#include <git2/revparse.h>
#include <git2/reset.h>
#include "TempRepo.hpp"
#include "Status.hpp"

TEST_CASE_METHOD(TempRepo, "Test on master up to date") {
    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch is up to date with 'origin/master'.\n"
            "\n" == stream.str());

}

TEST_CASE_METHOD(TempRepo, "Test local branch only") {
    branch("local_branch");

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    // For whatever reason git bash doesn't put a trailing newline when the branch is local and up
    // to date.
    REQUIRE("On branch local_branch\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test 1 new commit") {
    commit();
    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch is ahead of 'origin/master' by 1 commit.\n"
            "  (use \"git push\" to publish your local commits)\n"
            "\n" == stream.str());

}

TEST_CASE_METHOD(TempRepo, "Test 4 new commits") {
    commit();
    commit();
    commit();
    commit();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch is ahead of 'origin/master' by 4 commits.\n"
            "  (use \"git push\" to publish your local commits)\n"
            "\n" == stream.str());

}

TEST_CASE_METHOD(TempRepo, "Test 1 commit behind") {

    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~1");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch is behind 'origin/master' by 1 commit, and can be fast-forwarded.\n"
            "  (use \"git pull\" to update your local branch)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test 3 commits behind") {

    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~3");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch is behind 'origin/master' by 3 commits, and can be fast-forwarded.\n"
            "  (use \"git pull\" to update your local branch)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test branches diverged 3 behind and 1 forward") {

    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~3");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    commit();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch and 'origin/master' have diverged,\n"
            "and have 1 and 3 different commits each, respectively.\n"
            "  (use \"git pull\" to merge the remote branch into yours)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test branches diverged 2 behind and 4 forward") {

    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~2");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    commit();
    commit();
    commit();
    commit();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("On branch master\n"
            "Your branch and 'origin/master' have diverged,\n"
            "and have 4 and 2 different commits each, respectively.\n"
            "  (use \"git pull\" to merge the remote branch into yours)\n"
            "\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test detached state") {
    // The submodule paths are test specific to get to a repeatable sha we must jump back prior to the submodules.
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~2");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    git_reference * head = NULL;
    git_repository_head(&head, m_repo);
    auto oid = git_reference_target(head);
    git_repository_set_head_detached(m_repo, oid);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("HEAD detached at 92b4c41\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test detached state different commit", "[.not_implemented]") {
    // The submodule paths are test specific to get to a repeatable sha we must jump back prior to the submodules.
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~2");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    git_reference * head = NULL;
    git_repository_head(&head, m_repo);
    auto oid = git_reference_target(head);
    git_repository_set_head_detached(m_repo, oid);

    // move the head forward one
    commit();

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream);

    REQUIRE("HEAD detached from 92b4c41\n" == stream.str());
}

TEST_CASE_METHOD(TempRepo, "Test detached state with color") {
    // The submodule paths are test specific to get to a repeatable sha we must jump back prior to the submodules.
    git_object *object = NULL;
    git_revparse_single(&object, m_repo, "HEAD~2");
    git_checkout_options options = GIT_CHECKOUT_OPTIONS_INIT;
    git_reset(m_repo, object, GIT_RESET_HARD, &options);
    git_object_free(object);

    git_reference * head = NULL;
    git_repository_head(&head, m_repo);
    auto oid = git_reference_target(head);
    git_repository_set_head_detached(m_repo, oid);

    auto status = Status(m_repo);
    std::stringstream stream;
    status.getBranchMessage(stream, Colorize::COLORIZE);

    REQUIRE("\u001b[31mHEAD detached at\u001b[0m 92b4c41\n" == stream.str());
}

/*
On branch main
Your branch is behind 'origin/main' by 1 commit, and can be fast-forwarded.
  (use "git pull" to update your local branch)
 */

/*
On branch main
Your branch and 'origin/main' have diverged,
and have 1 and 1 different commits each, respectively.
  (use "git pull" to merge the remote branch into yours)

nothing to commit, working tree clean

 */

/* What about tags? */

/*
On branch branch_status
Unmerged paths:
  (use "git restore --staged <file>..." to unstage)
  (use "git add <file>..." to mark resolution)
        both modified:   tests/TestStatusBranch.cpp

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/win-git-status/Repo.cpp

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        foo.txt

no changes added to commit (use "git add" and/or "git commit -a")
 */