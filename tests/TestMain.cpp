//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#define CATCH_CONFIG_RUNNER

#include <catch2/catch.hpp>
#include <filesystem>
#include "TempDirectory.hpp"
#include "RepoBuilder.hpp"

void CreateSubmodule(const std::filesystem::path & path) {
    std::vector<std::string> files = {"file_1.txt", "file_2.txt", "file_3.txt"};
    auto repo = RepoBuilder(path.string());

    for(const auto & filename:files){
        auto full_path = path / filename;
        std::filesystem::create_directories(full_path.parent_path());
        auto file = std::ofstream(full_path);
        file << "Hello, World!\n";
        file.close();
        repo.addFile(filename);
    }
    repo.commit("Some message");
}

/// Creates a set of common remote repos that all of the tests can clone.
/// The layout will be as follows
///
///     main_repo
///         * - file_1.txt
///         * - file_2.txt
///         * - file_3.txt
///         * - sub_dir_1
///                 * - sub_1_file_1.txt
///                 * - sub_1_file_2.txt
///                 * - sub_1_file_3.txt
///         * - sub_dir_2
///                 * - sub_2_file_1.txt
///                 * - sub_2_file_2.txt
///                 * - sub_2_file_3.txt
///         * - sub_repo_1
///         * - sub_repo_2
///
/// The sub repos will have similar layouts of
///         * - sub_repo_1
///             * - file_1.txt
///             * - file_2.txt
///             * - file_3.txt
///         * - sub_repo_2
///             * - file_1.txt
///             * - file_2.txt
///             * - file_3.txt
///
/// The remotes will be sub directories in the provided `path`.
void CreateSessionRemotes(const std::filesystem::path & path) {
    std::vector<std::string> files = {"file_1.txt", "file_2.txt", "file_3.txt",
                                      "sub_dir_1/sub_1_file_1.txt", "sub_dir_1/sub_1_file_2.txt", "sub_dir_1/sub_1_file_3.txt",
                                      "sub_dir_2/sub_2_file_1.txt", "sub_dir_2/sub_2_file_2.txt", "sub_dir_2/sub_2_file_3.txt"};
    auto main_repo = path / "main_repo";
    RepoBuilder::setOriginRepo(main_repo.string());
    auto repo = RepoBuilder(main_repo.string());

    // Force stable line endings for some reason debug vs release provides different values.
    auto attribute_name = main_repo / ".gitattributes";
    auto attribute_file = std::ofstream(attribute_name);
    attribute_file << "*	text=auto\n";
    attribute_file.close();
    repo.addFile(".gitattributes");
    repo.commit("Adding .gitattributes");

    for(const auto & filename:files){
        auto full_path = main_repo / filename;
        std::filesystem::create_directories(full_path.parent_path());
        auto file = std::ofstream(full_path);
        file << "Hello, World!\n";
        file.close();
        repo.addFile(filename);
        repo.commit("Adding " + filename);
    }

    std::vector<std::string> sub_modules = {"sub_repo_1", "sub_repo_2"};
    for(const auto & sub_module:sub_modules) {
        auto sub_path = path / sub_module;
        CreateSubmodule(sub_path);
        repo.addSubmodule(sub_path.string(), sub_module);
        repo.commit("Add submodule " + sub_module);
    }
}

int main( int argc, char* argv[] ) {
    TempDirectory::SetIntermediateDir("git-win-status-tests");
    TempDirectory::Increment("the_tests_");
    CreateSessionRemotes(TempDirectory::GetFullBaseDir());

    int result = Catch::Session().run( argc, argv );

    return result;
}