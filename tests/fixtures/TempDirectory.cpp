//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

#include <filesystem>
#include <iostream>
#include <string>
#include "TempDirectory.hpp"
namespace fs = std::filesystem;

std::filesystem::path TempDirectory::s_intermediate_dir=std::filesystem::temp_directory_path();
std::filesystem::path TempDirectory::s_prefix_dir=TempDirectory::s_prefix_dir;

std::filesystem::path TempDirectory::TempDir(std::filesystem::path sub_dir) {
    auto temp_dir = s_prefix_dir;
    if(!sub_dir.empty()) {
        temp_dir /= sub_dir;
    }
    fs::create_directories(temp_dir);
    return temp_dir;
}

void TempDirectory::Increment(std::string prefix) {
    auto test_number = getNextTestNumber(prefix);
    auto base_dir = prefix + std::to_string(test_number);
    s_prefix_dir = s_intermediate_dir / base_dir;
    fs::create_directories(s_prefix_dir);
}

void TempDirectory::SetIntermediateDir(std::string intermediate_dir) {
    s_intermediate_dir /= intermediate_dir;
    fs::create_directories(s_intermediate_dir);
}

std::filesystem::path TempDirectory::GetFullBaseDir() {
    return s_prefix_dir;
}

int TempDirectory::getNextTestNumber(std::string base_dir) {
    auto && directories = std::vector<fs::directory_entry>{};
    auto iterator = fs::directory_iterator(s_intermediate_dir);
    copy_if(begin(iterator), end(iterator), std::back_inserter(directories),
            [base_dir](auto dir) { return dir.is_directory() && (dir.path().filename().string().rfind(base_dir, 0) == 0); });

    std::vector<int> suffixes;
    std::transform(directories.begin(), directories.end(), std::back_inserter(suffixes),
                   [base_dir](auto dir) {
                        auto suffix = dir.path().filename().string().substr(base_dir.length());
                        try {
                            return std::stoi(suffix);
                        }
                        catch(std::invalid_argument){
                            return 0;
                        }});

    std::vector<int> sorted;
    std::copy(suffixes.begin(), suffixes.end(), std::back_inserter(sorted));
    std::sort(sorted.begin(), sorted.end());

    int largest = 0;
    if (!sorted.empty()) {
        largest = sorted.back();
    }
    return largest + 1;
}

