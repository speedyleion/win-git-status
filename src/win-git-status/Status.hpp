//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#pragma once

#include <iostream>

#include <git2/types.h>
#include <git2/status.h>

class Status {
public:
    Status(git_repository * repo);
    ~Status();
    void toStream(std::ostream &stream);
    bool getUntrackedMessage(std::ostream &stream);

    bool getBranchMessage(std::ostream &stream);

    bool getTrackedMessage(std::ostream &stream);

    bool getStagedMessage(std::ostream &stream);

    bool
    getStatusMessage(std::ostream &stream, const std::string &header, int group_status, size_t diff_offset);

    std::string getFileMessage(git_status_t status, const git_diff_delta *file_diff);

private:
    git_status_list *m_status;
    git_repository *m_repo;
    bool m_unstaged_submodule = false;
};


