/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */


use crate::{DirEntry, Index};
use std::path::Path;
use git2::{Repository, TreeWalkMode, TreeWalkResult, TreeEntry, ObjectType};
use crate::direntry::FileStat;
use std::convert::TryInto;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    CURRENT,
    NEW,
    MODIFIED,
    DELETED,
}

impl Default for Status {
    fn default() -> Self {
        Status::CURRENT
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct TreeDiffEntry {
    pub name: String,
    pub state: Status,
}

/// A tree of a repo.
///
#[derive(Debug, Default, PartialEq)]
pub struct TreeDiff {
    path: String,
    pub entries: Vec<TreeDiffEntry>,
}

impl TreeDiff {
    pub fn diff_against_index(path: &Path) -> TreeDiff {
        TreeDiff::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;
    use temp_testdir::TempDir;

    #[test]
    fn test_get_tree_diff() {
        let temp_dir = TempDir::default();
        assert_eq!(TreeDiff::diff_against_index(&temp_dir), TreeDiff::default());
    }

}
