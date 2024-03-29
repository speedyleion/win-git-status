/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ObjectType {
    Regular,
    SymLink,
    // These are submodules
    GitLink,
}
impl Default for ObjectType {
    fn default() -> Self {
        ObjectType::Regular
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct FileStat {
    // modified time in nanoseconds since the unix epoch
    pub mtime: u128,
    pub size: u32,
}

/// Represents an git entry in the index or working tree i.e. a file or blob
#[derive(PartialEq, Eq, Debug, Default)]
pub struct DirEntry {
    pub object_type: ObjectType,
    pub stat: FileStat,

    // The docs call this "object name"
    pub sha: [u8; 20],
    pub name: String,
}
