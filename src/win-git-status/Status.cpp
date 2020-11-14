//          Copyright Nick G 2020.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
#include <git2/status.h>
#include <git2/submodule.h>
#include <sstream>
#include <git2/branch.h>
#include <git2/graph.h>
#include "Status.hpp"

Status::Status(git_repository *repo) : m_repo(repo) {
    git_status_options options = GIT_STATUS_OPTIONS_INIT;
    options.flags |= GIT_STATUS_OPT_INCLUDE_UNTRACKED | GIT_STATUS_OPT_RENAMES_HEAD_TO_INDEX;
    git_status_list_new(&m_status, m_repo, &options);
}

Status::~Status() {
    git_status_list_free(m_status);
}

void Status::toStream(std::ostream &stream, Colorize colorize) {
    getBranchMessage(stream, colorize);
    auto tracked_message = getTrackedMessage(stream, colorize);
    auto staged_message = getStagedMessage(stream, colorize);
    auto untracked_message = getUntrackedMessage(stream, colorize);

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

bool Status::getBranchMessage(std::ostream &stream, Colorize colorize) {
    git_reference * branch = NULL;
    git_repository_head(&branch, m_repo);
    const char * branch_name = NULL;

    if(git_branch_name(&branch_name, branch) != 0) {

        std::string color;
        std::string color_end;
        if (colorize == Colorize::COLORIZE) {
            color = "\u001b[31m";
            color_end = "\u001b[0m";
        }
        auto oid = git_reference_target(branch);
        git_object * object = NULL;
        git_object_lookup(&object, m_repo, oid, GIT_OBJECT_COMMIT);
        git_buf buffer = {0};
        git_object_short_id(&buffer, object);
        stream << color << "HEAD detached at" << color_end << " " << buffer.ptr << "\n";
        git_reference_free(branch);
        git_object_free(object);
        return true;
    }

    stream << "On branch " << branch_name << "\n";

    git_reference * upstream = NULL;
    if(git_branch_upstream(&upstream, branch) == 0){
        const char * upstream_name = NULL;
        git_branch_name(&upstream_name, upstream);

        size_t ahead = 0;
        size_t behind = 0;
        auto local_oid = git_reference_target(branch);
        auto upstream_oid = git_reference_target(upstream);
        git_graph_ahead_behind(&ahead, &behind, m_repo, local_oid, upstream_oid);

        if(ahead && behind) {
            stream << "Your branch and '" << upstream_name << "' have diverged,\n";
            stream << "and have " << ahead << " and " << behind << " different commits each, respectively.\n";
            stream << "  (use \"git pull\" to merge the remote branch into yours)\n";
        }
        else if(ahead) {
            std::string plural = ahead == 1 ? "" : "s";
            stream << "Your branch is ahead of '" << upstream_name << "' by " << ahead << " commit" << plural << ".\n";
            stream << "  (use \"git push\" to publish your local commits)\n";
        }
        else if(behind) {
            std::string plural = behind == 1 ? "" : "s";
            stream << "Your branch is behind '" << upstream_name << "' by " << behind << " commit" << plural << ", and can be fast-forwarded.\n";
            stream << "  (use \"git pull\" to update your local branch)\n";
        }
        else{
            stream << "Your branch is up to date with '" << upstream_name << "'.\n";
        }

        stream << "\n";

        git_reference_free(upstream);
    }

    git_reference_free(branch);

    return true;
}

bool Status::getUntrackedMessage(std::ostream &stream, Colorize colorize) {
    std::string header = "Untracked files:\n"
                         "  (use \"git add <file>...\" to include in what will be committed)\n";
    std::string color;
    if (colorize == Colorize::COLORIZE) {
    color = "\u001b[31m";
    }
    return getStatusMessage(stream, header, GIT_STATUS_WT_NEW, offsetof(git_status_entry, index_to_workdir), color);
}

bool Status::getTrackedMessage(std::ostream &stream, Colorize colorize) {
    std::string header = "Changes not staged for commit:\n"
                         "  (use \"git add <file>...\" to update what will be committed)\n"
                         "  (use \"git restore <file>...\" to discard changes in working directory)\n";

    std::string submodule_message = "  (commit or discard the untracked or modified content in submodules)\n";

    std::string color;
    if (colorize == Colorize::COLORIZE) {
        color = "\u001b[31m";
    }

    // For submodules we need to inject a message in between so must use a local string stream to cache up the file results
    std::stringstream local_stream;
    auto has_unstaged = getStatusMessage(local_stream, "", (GIT_STATUS_WT_MODIFIED | GIT_STATUS_WT_DELETED),
                                         offsetof(git_status_entry, index_to_workdir), color);
    if(has_unstaged){
        stream << header;
        if(m_unstaged_submodule){
            stream << submodule_message;
        }
        stream << local_stream.str();
    }
    return has_unstaged;
}

bool Status::getStagedMessage(std::ostream &stream, Colorize colorize) {
    std::string header = "Changes to be committed:\n"
                         "  (use \"git restore --staged <file>...\" to unstage)\n";
    std::string color;
    if (colorize == Colorize::COLORIZE) {
        color = "\u001b[32m";
    }

    return getStatusMessage(stream, header,
                            (GIT_STATUS_INDEX_NEW | GIT_STATUS_INDEX_RENAMED | GIT_STATUS_INDEX_MODIFIED |
                             GIT_STATUS_INDEX_DELETED), offsetof(git_status_entry, head_to_index), color);

}

bool
Status::getStatusMessage(std::ostream &stream, const std::string &header, int group_status, size_t diff_offset,
                         const std::string &file_color) {
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
            stream << "        " << getFileMessage((git_status_t) (entry->status & group_status), (*file_delta),
                                                   file_color);
        }
    }
    if(entries_found){
        stream << "\n";
    }

    return entries_found;
}

std::string
Status::getFileMessage(git_status_t status, const git_diff_delta *file_diff, const std::string &file_color) {
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
    std::string epilog;
    if(status & GIT_STATUS_WT_MODIFIED) {
        // Trying for possible submodule state
        unsigned int sub_status;
        if(git_submodule_status(&sub_status, m_repo, file_diff->old_file.path, GIT_SUBMODULE_IGNORE_NONE) == 0){
            if(sub_status & GIT_SUBMODULE_STATUS_WD_MODIFIED){
                m_unstaged_submodule = true;
                epilog += "new commits";
            }
            if(sub_status & (GIT_SUBMODULE_STATUS_WD_WD_MODIFIED | GIT_SUBMODULE_STATUS_WD_INDEX_MODIFIED)){
                m_unstaged_submodule = true;
                if(!epilog.empty()){
                    epilog += ", ";
                }
                epilog += "modified content";
            }
            if(sub_status & GIT_SUBMODULE_STATUS_WD_UNTRACKED){
                m_unstaged_submodule = true;
                if(!epilog.empty()){
                    epilog += ", ";
                }
                epilog += "untracked content";
            }
            epilog = std::string(" (") + epilog + ")";
        }
    }
    std::string end_color = file_color.empty() ? "" : "\u001b[0m";
    return file_color + change_type + file + end_color + epilog + "\n";
}
