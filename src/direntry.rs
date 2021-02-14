/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

/// Represents an git entry in the index or working tree i.e. a file or blob
#[derive(PartialEq, Eq, Debug)]
pub struct DirEntry {
    // The docs call this "object name"
    pub sha: [u8; 20],
    pub name: String,
}

