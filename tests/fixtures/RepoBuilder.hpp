//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#pragma once

#include <filesystem>
#include "git2/repository.h"

class RepoBuilder {
public:
    RepoBuilder(const std::string &path);
    ~RepoBuilder();

    void addFile(std::string filename);
    void addSubmodule(const std::string &url, const std::string &path);

    void commit(std::string message);
    static void setOriginRepo(const std::string &url);
    static const std::string getOriginRepo();

private:
    git_repository * m_repo;
    int m_time=0;
    static std::string m_origin;
};

