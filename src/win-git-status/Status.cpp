//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#include <git2/status.h>
#include <git2/submodule.h>
#include <sstream>
#include "Status.hpp"

Status::Status(git_repository *repo) : m_repo(repo) {
    git_status_options options = GIT_STATUS_OPTIONS_INIT;
    options.flags |= GIT_STATUS_OPT_INCLUDE_UNTRACKED | GIT_STATUS_OPT_RENAMES_HEAD_TO_INDEX;
    git_status_list_new(&m_status, m_repo, &options);
}

Status::~Status() {
    git_status_list_free(m_status);
}

void Status::toStream(std::ostream &stream) {
    getBranchMessage(stream);
    auto tracked_message = getTrackedMessage(stream);
    auto staged_message = getStagedMessage(stream);
    auto untracked_message = getUntrackedMessage(stream);

    if (tracked_message) {
        stream << std::string("no changes added to commit (use \"git add\" and/or \"git commit -a\")\n");
    } else if(staged_message){
        // do nothing
    } else if(untracked_message){
        stream << std::string("nothing added to commit but untracked files present (use \"git add\" to track)\n");
    } else {
        stream << std::string("nothing to commit, working tree clean\n");
    }
}

bool Status::getBranchMessage(std::ostream &stream) {
    std::string message;
    message = "On branch master\n"
              "Your branch is up to date with 'origin/master'.\n"
              "\n";

    stream << message;

    return true;
}

bool Status::getUntrackedMessage(std::ostream &stream) {
    std::string header = "Untracked files:\n"
                         "  (use \"git add <file>...\" to include in what will be committed)\n";
    return getStatusMessage(stream, header, GIT_STATUS_WT_NEW, offsetof(git_status_entry, index_to_workdir));
}

bool Status::getTrackedMessage(std::ostream &stream) {
    std::string header = "Changes not staged for commit:\n"
                         "  (use \"git add <file>...\" to update what will be committed)\n"
                         "  (use \"git restore <file>...\" to discard changes in working directory)\n";
    std::string submodule_message = "  (commit or discard the untracked or modified content in submodules)\n";

    // For submodules we need to inject a message in between so must use a local string stream to cache up the file results
    std::stringstream local_stream;
    auto has_unstaged = getStatusMessage(local_stream, "", (GIT_STATUS_WT_MODIFIED | GIT_STATUS_WT_DELETED),
                                         offsetof(git_status_entry, index_to_workdir));
    if(has_unstaged){
        stream << header;
        if(m_unstaged_submodule){
            stream << submodule_message;
        }
        stream << local_stream.str();
    }
    return has_unstaged;
}

bool Status::getStagedMessage(std::ostream &stream) {
    std::string header = "Changes to be committed:\n"
                         "  (use \"git restore --staged <file>...\" to unstage)\n";
    return getStatusMessage(stream, header,
                            (GIT_STATUS_INDEX_NEW | GIT_STATUS_INDEX_RENAMED | GIT_STATUS_INDEX_MODIFIED |
                             GIT_STATUS_INDEX_DELETED), offsetof(git_status_entry, head_to_index));

}

bool
Status::getStatusMessage(std::ostream &stream, const std::string &header, int group_status, size_t diff_offset) {
    auto num_entries = git_status_list_entrycount(m_status);
    std::string message;
    bool entries_found = false;
    if(!num_entries){
        return entries_found;
    }

    for(decltype(num_entries) i=0; i < num_entries; i++) {
        const git_status_entry *entry;
        entry = git_status_byindex(m_status, i);
        if (entry->status & group_status) {
            if(!entries_found) {
                entries_found = true;
                stream << header;
            }
            auto **file_delta = (git_diff_delta **) ((uint8_t *) entry + diff_offset);
            stream << getFileMessage((git_status_t)(entry->status & group_status), (*file_delta));
        }
    }
    if(entries_found){
        stream << "\n";
    }

    return entries_found;
}

std::string Status::getFileMessage(git_status_t status, const git_diff_delta *file_diff) {
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
    if(status & GIT_STATUS_WT_MODIFIED) {
        // Trying for possible submodule state
        unsigned int sub_status;
        if(git_submodule_status(&sub_status, m_repo, file_diff->old_file.path, GIT_SUBMODULE_IGNORE_NONE) == 0){
            if(sub_status & GIT_SUBMODULE_STATUS_WD_UNTRACKED){
                file += std::string(" (untracked content)");
                m_unstaged_submodule = true;
            }
        }
    }
    return std::string("        ") + change_type + file + "\n";
}
