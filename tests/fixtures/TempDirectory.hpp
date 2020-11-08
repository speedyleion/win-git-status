//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#pragma once

#include <filesystem>

/// Provide functionality to allow tests to utilize temporary directories which will persist past a test run while still
/// having these directories being cleaned up later.
///
/// The intended functionality is modeled after that of the tempdir in pytest.  In particular
/// https://docs.pytest.org/en/stable/tmpdir.html#the-default-base-temporary-directory
///
/// For example on Windows this would provide one with a path like
///
///     "C:\\Users\\<username>\\AppData\\Local\\Temp\\<intermediate_dir>\\<base>_###\\<sub_dir>"
///
/// - the `intermediate_dir` is set via TempDirectory::SetIntermediateDir()
/// - the `base` is set via TempDirectory::Increment()
/// - the `sub_dir` is appended when calling TempDirectory::TempDir()
class TempDirectory {

public:
    /// Creates and returns a temporary directory.
    static std::filesystem::path TempDir(std::filesystem::path sub_dir="sub_dir");

    /// Increments the `base` portion of the temporary directories and creates the base directory.
    /// If there are more than `rolling_count` base directories the lower value will be removed.
    static void Increment(std::string base="base");

    /// Sets the intermediate portion of the temporary directories.  Creates the intermediate directory if it didn't
    /// exist.
    static void SetIntermediateDir(std::string intermediate_dir="intermediate_dir");

    static std::filesystem::path GetFullBaseDir();

    const static int rolling_count = 3;

private:
    static std::filesystem::path s_intermediate_dir;
    static std::filesystem::path s_prefix_dir;

    static int getNextTestNumber(std::string base_dir);
};