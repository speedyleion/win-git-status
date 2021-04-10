/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
mod direntry;
mod dirstat;
mod error;
mod index;
mod repo_status;
pub mod status;
mod tree;
pub mod worktree;

pub use direntry::DirEntry;
pub use error::StatusError;
pub use index::Index;
pub use repo_status::RepoStatus;
pub use tree::TreeDiff;
pub use worktree::WorkTree;
