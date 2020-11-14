//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#ifndef WIN_GIT_STATUS_TEMPREPO_HPP
#pragma once

#include <filesystem>
#include "git2/types.h"

class TempRepo {
public:
    TempRepo();
    ~TempRepo();
    void addFile(const std::string &filename, const std::string &submodule_path="");
    void commit(const std::string &submodule_path="");
    void removeFile(const std::string &filename);

protected:
    std::filesystem::path m_dir;
    git_repository *m_repo;
};