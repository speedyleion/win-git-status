/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::{WorkTree, TreeDiff, Index};
use std::path::Path;
use crate::error::StatusError;

#[derive(Debug)]
pub struct RepoStatus {
    index_diff: TreeDiff,
    work_tree_diff: WorkTree,
}

impl RepoStatus {
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<RepoStatus, StatusError> {
        let index_file = path.join(".git/index");
        let index = Index::new(&*index_file)?;
        let work_tree_diff = WorkTree::diff_against_index(path, index)?;
        let index_diff = TreeDiff::diff_against_index(path);
        Ok(RepoStatus { index_diff, work_tree_diff })
    }
}
