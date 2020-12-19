//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <catch2/catch.hpp>
#include <Status.hpp>
#include <sstream>
#include <fstream>
#include "TempRepo.hpp"

TEST_CASE_METHOD(TempRepo, "Test no staged changes") {
    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_message = status.getStagedMessage(stream);

    REQUIRE("" == stream.str());
    REQUIRE(has_message == false);
}

TEST_CASE_METHOD(TempRepo, "Test staged file deleted") {
    removeFile("file_2.txt");
    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_message = status.getStagedMessage(stream);

    REQUIRE("Changes to be committed:\n"
            "  (use \"git restore --staged <file>...\" to unstage)\n"
            "        deleted:    file_2.txt\n"
            "\n" == stream.str());

    REQUIRE(has_message == true);

}

TEST_CASE_METHOD(TempRepo, "Test staged changes while in a merge state") {
    merge("origin/master");

    auto file_to_modify = m_dir / "file_1.txt";
    auto file = std::ofstream(file_to_modify);
    file << "This file is modified\n";
    file.close();
    addFile(file_to_modify);

    auto status = Status(m_repo);
    std::stringstream stream;
    auto has_message = status.getStagedMessage(stream);

    REQUIRE("Changes to be committed:\n"
            "        modified:   file_1.txt\n"
            "\n" == stream.str());

    REQUIRE(has_message == true);

}
