//          Copyright Nick G 2020
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)#include "Repo.hpp"

#include <git2/global.h>
#include <git2/repository.h>
#include <git2/status.h>
#include "Repo.hpp"

Repo::Repo(const std::string &path) {
    git_libgit2_init();

    m_repo = NULL;
    git_repository_open(&m_repo, path.c_str());
}

Repo::~Repo(){
    git_repository_free(m_repo);
}

std::string Repo::status() {
    git_status_list *status = NULL;
    git_status_options options = GIT_STATUS_OPTIONS_INIT;
    options.flags |= GIT_STATUS_OPT_INCLUDE_UNTRACKED | GIT_STATUS_OPT_RENAMES_HEAD_TO_INDEX;
    git_status_list_new(&status, m_repo, &options);

    auto branch_message = getBranchMessage(status);
    auto tracked_message = getTrackedMessage(status);
    auto untracked_message = getUntrackedMessage(status);
    auto staged_message = getStagedMessage(status);

    git_status_list_free(status);

    std::string full_message = branch_message + tracked_message + staged_message + untracked_message;

    if (!tracked_message.empty()) {
        full_message += std::string("no changes added to commit (use \"git add\" and/or \"git commit -a\")\n");
    } else if(!staged_message.empty()){
        // do nothing
    } else if(!untracked_message.empty()){
        full_message += std::string("nothing added to commit but untracked files present (use \"git add\" to track)\n");
    } else {
        full_message += std::string("nothing to commit, working tree clean\n");
    }
    return full_message;
}

std::string Repo::getBranchMessage(git_status_list *status) {
    std::string message;
    message = "On branch master\n"
              "Your branch is up to date with 'origin/master'.\n"
              "\n";

    return message;
}

std::string Repo::getUntrackedMessage(git_status_list *status) {
    std::string header = "Untracked files:\n"
                         "  (use \"git add <file>...\" to include in what will be committed)\n";
    return getStatusMessage(status, header, GIT_STATUS_WT_NEW, offsetof(git_status_entry, index_to_workdir));
}

std::string Repo::getTrackedMessage(git_status_list *status) {
    std::string header = "Changes not staged for commit:\n"
                         "  (use \"git add <file>...\" to update what will be committed)\n"
                         "  (use \"git restore <file>...\" to discard changes in working directory)\n";
    return getStatusMessage(status, header, (GIT_STATUS_WT_MODIFIED | GIT_STATUS_WT_DELETED), offsetof(git_status_entry, index_to_workdir));
}

std::string Repo::getStagedMessage(git_status_list *status) {
    std::string header = "Changes to be committed:\n"
                         "  (use \"git restore --staged <file>...\" to unstage)\n";
    return getStatusMessage(status, header, (GIT_STATUS_INDEX_NEW | GIT_STATUS_INDEX_RENAMED | GIT_STATUS_INDEX_MODIFIED | GIT_STATUS_INDEX_DELETED), offsetof(git_status_entry, head_to_index));

}

std::string Repo::getStatusMessage(git_status_list *status, const std::string &header, int group_status,
                                   size_t diff_offset) const {
    auto num_entries = git_status_list_entrycount(status);
    std::string message;
    if(!num_entries){
        return message;
    }

    for(decltype(num_entries) i=0; i < num_entries; i++) {
        const git_status_entry *entry;
        entry = git_status_byindex(status, i);
        if (entry->status & group_status) {
            git_diff_delta **file_delta = (git_diff_delta **) ((uint8_t *) entry + diff_offset);
            message += getFileMessage((git_status_t)(entry->status & group_status), (*file_delta));
        }
    }

    if(!message.empty()){
        return header + message + "\n";
    }
    return message;
}

std::string Repo::getFileMessage(git_status_t status, const git_diff_delta *file_diff) const {
    std::string change_type;
    if(status & (GIT_STATUS_INDEX_MODIFIED | GIT_STATUS_WT_MODIFIED)) {
        change_type = "modified:   ";
    } else if(status & (GIT_STATUS_INDEX_RENAMED | GIT_STATUS_WT_RENAMED)) {
        change_type = "renamed:    ";
    } else if(status & (GIT_STATUS_INDEX_DELETED | GIT_STATUS_WT_DELETED)) {
        change_type = "deleted:    ";
    } else if(status & GIT_STATUS_INDEX_NEW) {
        // GIT_STATUS_WT_NEW is intentionally not here, it goes with untracked files and doesn't get a file decorator.
        change_type = "new file:   ";
    }

    std::string file(file_diff->old_file.path);
    if(status & (GIT_STATUS_INDEX_RENAMED | GIT_STATUS_WT_RENAMED)) {
        file += std::string(" -> ") + file_diff->new_file.path;
    }
    return std::string("        ") + change_type + file + "\n";
}

