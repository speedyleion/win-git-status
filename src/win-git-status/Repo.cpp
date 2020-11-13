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

std::string Repo::getUntrackedMessage(git_status_list *status) {
    auto num_entries = git_status_list_entrycount(status);
    std::string message;
    if(!num_entries){
        return message;
    }
    for(decltype(num_entries) i=0; i < num_entries; i++){
        const git_status_entry * entry;
        entry = git_status_byindex(status, i);
        if(entry->status & GIT_STATUS_WT_NEW){
            if(message.empty()){
                message = "Untracked files:\n"
                          "  (use \"git add <file>...\" to include in what will be committed)\n";
            }
            message += std::string("        ") + entry->index_to_workdir->old_file.path + "\n";
        }

    }
    if(!message.empty()){
        message += "\n";
    }
    return message;
}

std::string Repo::getBranchMessage(git_status_list *status) {
    std::string message;
    message = "On branch master\n"
              "Your branch is up to date with 'origin/master'.\n"
              "\n";

    return message;
}

std::string Repo::getTrackedMessage(git_status_list *status) {
    auto num_entries = git_status_list_entrycount(status);
    std::string message;
    if(!num_entries){
        return message;
    }

    for(decltype(num_entries) i=0; i < num_entries; i++){
        const git_status_entry * entry;
        entry = git_status_byindex(status, i);
        if(entry->status & (GIT_STATUS_WT_MODIFIED | GIT_STATUS_WT_DELETED)){

            std::string change_type;
            if(entry->status & GIT_STATUS_WT_MODIFIED) {
                change_type = "modified:";
            } else {
                change_type = "deleted: ";
            }
            if(message.empty()) {
                message = "Changes not staged for commit:\n"
                          "  (use \"git add <file>...\" to update what will be committed)\n"
                          "  (use \"git restore <file>...\" to discard changes in working directory)\n";
            }
            message += std::string("        ") + change_type + "   " + entry->index_to_workdir->old_file.path + "\n";
        }

    }
    if(!message.empty()){
        message += "\n";
    }
    return message;
}

std::string Repo::getStagedMessage(git_status_list *status) {
    auto num_entries = git_status_list_entrycount(status);
    std::string message;
    if(!num_entries){
        return message;
    }

    for(decltype(num_entries) i=0; i < num_entries; i++){
        const git_status_entry * entry;
        entry = git_status_byindex(status, i);
        if(entry->status & (GIT_STATUS_INDEX_NEW | GIT_STATUS_INDEX_RENAMED | GIT_STATUS_INDEX_MODIFIED | GIT_STATUS_INDEX_DELETED)){

            if(message.empty()) {
                message = "Changes to be committed:\n"
                          "  (use \"git restore --staged <file>...\" to unstage)\n";
            }

            std::string change_type;
            if(entry->status & GIT_STATUS_INDEX_MODIFIED) {
                change_type = "modified:";
            } else if(entry->status & GIT_STATUS_INDEX_RENAMED) {
                change_type = "renamed: ";
            } else if(entry->status & GIT_STATUS_INDEX_DELETED) {
                change_type = "deleted: ";
            } else {
                change_type = "new file:";
            }

            std::string file(entry->head_to_index->old_file.path);
            if(entry->status == GIT_STATUS_INDEX_RENAMED) {
                file += std::string(" -> ") + entry->head_to_index->new_file.path;
            }
            message += std::string("        ") + change_type + "   " + file + "\n";
        }
    }
    if(!message.empty()){
        message += "\n";
    }
    return message;
}

