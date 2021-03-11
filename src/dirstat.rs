/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;
use std::collections::HashMap;
use ntapi::ntioapi::{NtQueryDirectoryFile, IO_STATUS_BLOCK, FileFullDirectoryInformation};
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
use std::ffi::CString;
use winapi::um::winnt::{FILE_LIST_DIRECTORY, FILE_SHARE_DELETE, HANDLE, FILE_SHARE_WRITE, FILE_SHARE_READ};
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct FileStat {
    pub mtime: u32,
    pub size: u32,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct DirectoryStat {
    pub directory: String,
    pub file_stats: HashMap<String, FileStat>,
}

impl DirectoryStat {
    /// Returns the index for the git repo at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to a directory to get file stats fro
    pub fn new(path: &Path) -> DirectoryStat {
        let mut file_stats = DirectoryStat::get_dir_stats(path);


        let dirstat = DirectoryStat{directory: path.to_str().unwrap().to_string(), file_stats};
        dirstat
    }

    fn get_dir_stats(path: &Path) -> HashMap<String, FileStat> {
        let mut file_stats = HashMap::new();
        let handle = DirectoryStat::get_directory_handle(path);
        let mut io_block: IO_STATUS_BLOCK = unsafe {std::mem::zeroed()};
        let io_ptr: *mut IO_STATUS_BLOCK = &mut io_block as *mut _;
        let foo = unsafe { NtQueryDirectoryFile(handle, std::ptr::null_mut(), None, std::ptr::null_mut(), io_ptr, buffer, buffer_size,
                                                FileFullDirectoryInformation, 1, std::ptr::null_mut(), 0)};
        file_stats
    }

    fn get_directory_handle(path: &Path) -> HANDLE {
        let name= CString::new(path.to_str().unwrap()).unwrap();
        let foo = unsafe {CreateFileA(name.as_ptr(), FILE_LIST_DIRECTORY, FILE_SHARE_WRITE | FILE_SHARE_READ | FILE_SHARE_DELETE, std::ptr::null_mut(), OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, std::ptr::null_mut())};
        foo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_testdir::TempDir;
    use std::fs;

    // Test helper function to build up a temporary directory of `files`.  All files will have the
    // contents of their name.
    fn temp_tree(files: Vec<&Path>) -> TempDir {
        let temp_dir = TempDir::default();

        for file in files {
            let full_path = temp_dir.join(file);
            fs::write(&full_path, file.to_str().unwrap()).unwrap();
        }
        temp_dir
    }

    #[test]
    fn test_one_entry_in_dir_stat() {

        let names = vec!["one"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = temp_tree(files);

        let dirstat = DirectoryStat::new(&temp_dir);
        assert_eq!(
            dirstat.file_stats.len(),
            1
        );
    }

    #[test]
    fn test_no_entries_in_dir_stat() {
        let temp_dir = temp_tree(vec![]);

        let dirstat = DirectoryStat::new(&temp_dir);
        assert_eq!(
            dirstat.file_stats.len(),
            0
        );
    }
}