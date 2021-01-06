/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;

/// An index of a repo.
/// Some refer to this as the cache or staging area.
///
/// This is meant to be a representation of the git index file.  The documentation for this format
/// can be found https://git-scm.com/docs/index-format.
///
/// Some common git internal terms.
///
/// - `oid` - Object ID.  This is often the SHA of an item.  It could be a commit, file blob, tree,
///     etc.
pub struct Index {
    path: String,
    oid: [u8; 20],
}

struct Header {
    version: u32,
    entries: u32,
}

impl Index {
    /// Returns the index for the git repo at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Index {
        let oid = [
            75, 130, 93, 198, 66, 203, 110, 185, 160, 96, 229, 75, 248, 214, 146, 136, 251, 238,
            73, 4,
        ];
        let index = Index {
            path: String::from(path.to_str().unwrap()),
            oid,
        };
        //HACK quiet warning for now
        assert!(index.path != "foo");
        index
    }

    /// Returns the oid(Object ID) for the index.
    ///
    /// The object ID of an index is the object ID of the tree which the index represents.
    pub fn oid(&self) -> &[u8] {
        &self.oid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_header() {
        let version: u32 = 2;
        let entries: u32 = 3;
        let mut header: Vec<u8> = vec![];
        header.extend(b"DIRC");
        header.extend(&version.to_be_bytes());
        header.extend(&entries.to_be_bytes());
        assert_eq!(Index::read_header(header), Header{version, entries});
    }
}
