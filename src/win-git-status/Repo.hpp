//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#pragma once


#include <git2/types.h>
#include <git2/status.h>
#include <string>

class Repo {
public:
    Repo::Repo(const std::string &path);
    Repo::~Repo();

    std::string status();

private:
    git_repository * m_repo;

};


