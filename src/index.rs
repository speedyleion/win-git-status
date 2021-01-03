/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;
use std::error::Error;

/// An index of a repo
pub struct Index {
    path: String,
    oid: [u8; 26]
}

impl Index {
    /// Create a new index from an on disk
    ///
    /// Returns error if the index file doesn't exist for a repo
    pub fn new(path: &Path) -> Index {
        let oid = [0; 26];
        let index = Index{path: String::from(path.to_str().unwrap()), oid};
        index
    }
    pub fn oid(&self) -> &[u8] {
        &self.oid
    }
}
