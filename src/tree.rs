/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */


use crate::DirEntry;
use std::path::Path;

/// A tree of a repo.
///
/// Some common git internal terms.
///
#[derive(Debug, Default, PartialEq)]
pub struct Tree {
    path: String,
    pub trees: Vec<Tree>,
    pub files: Vec<DirEntry>
}

impl Tree {
    pub fn new(path: &Path) -> Tree {
        Tree::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tree() {
        assert_eq!(Tree::new(Path::new("what")), Tree::default());
    }

}
