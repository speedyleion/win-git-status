/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;
use std::fs;
use sha1::{Sha1, Digest};

#[derive(Debug)]
pub struct DirEntryError {
    message: String,
}

/// Represents an git entry in the index or working tree i.e. a file or blob
#[derive(PartialEq, Eq, Debug)]
pub struct DirEntry {
    // The docs call this "object name"
    pub sha: [u8; 20],
    pub name: String,
}

impl DirEntry {
    /// Returns the index for the git repo at `path`.
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the git repo.
    /// * `name` - The name of the entry, relative to `root`.
    pub fn from_path(_root: &Path, name: &Path) -> Result<DirEntry, DirEntryError> {
        let string_name = name.to_str().ok_or_else(|| DirEntryError{message: "Failure to get string of file path".to_string()})?.to_string();
        Ok(DirEntry{sha: DirEntry::hash_file(name)?, name: string_name})
    }

    fn hash_file(_file: &Path) -> Result<[u8; 20], DirEntryError> {
        let result = Sha1::digest(b"what\nis\nit");
        let hash: [u8; 20] = result.into();
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_testdir::TempDir;

    #[test]
    fn test_from_path_1() {
        let temp_dir = TempDir::default();
        let file_contents = "what\r\nis\r\nit";
        let entry_name = "a/nested/file.txt";
        let file = temp_dir.join(entry_name);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, file_contents).unwrap();

        let entry = DirEntry::from_path(&temp_dir, Path::new(entry_name)).unwrap();
        assert_eq!(entry, DirEntry{sha: *b"00000000000000000000", name: entry_name.to_string()});

    }

    #[test]
    fn test_hash_file_removes_carriage_returns() {
        let temp_dir = TempDir::default();
        let file_contents = "what\r\nis\r\nit";
        let file = temp_dir.join("my_hash_file.txt");
        fs::write(file.clone(), file_contents).unwrap();

        let actual = DirEntry::hash_file(&file).unwrap();
        // let expected: [u8; 20] = Sha1::digest(file_contents.as_bytes()).into();
        let expected: [u8; 20] = Sha1::digest("what\nis\nit".as_bytes()).into();
        assert_eq!(actual, expected);

    }

    #[test]
    fn test_hash_file_leaves_newlines_alone() {
        let temp_dir = TempDir::default();
        let file_contents = "some\nother\nstring\ncontents";
        let file = temp_dir.join("some_other_file.txt");
        fs::write(file.clone(), file_contents).unwrap();

        let actual = DirEntry::hash_file(&file).unwrap();
        let expected: [u8; 20] = Sha1::digest(file_contents.as_bytes()).into();
        assert_eq!(actual, expected);

    }
}