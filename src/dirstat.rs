/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;
use std::collections::HashMap;
use ntapi::ntioapi::{NtQueryDirectoryFile, IO_STATUS_BLOCK, FileFullDirectoryInformation, FILE_FULL_DIR_INFORMATION};
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
use std::ffi::CString;
use winapi::um::winnt::{FILE_LIST_DIRECTORY, FILE_SHARE_DELETE, HANDLE, FILE_SHARE_WRITE, FILE_SHARE_READ, FILE_ATTRIBUTE_DIRECTORY};
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
use winapi::um::handleapi::CloseHandle;
use memoffset::offset_of;

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
        println!{"{:?}", path};
        let mut file_stats = HashMap::new();
        let handle = DirectoryStat::get_directory_handle(path);
        let mut io_block: IO_STATUS_BLOCK = unsafe {std::mem::zeroed()};
        let io_ptr: *mut IO_STATUS_BLOCK = &mut io_block as *mut _;
        let mut buffer: [u8; 1000] = [0; 1000];
        let name_member_offset = offset_of!(FILE_FULL_DIR_INFORMATION, FileName);
        loop{
            let mut offset = 0;
            let result = unsafe { NtQueryDirectoryFile(handle, std::ptr::null_mut(), None, std::ptr::null_mut(), io_ptr, buffer.as_mut_ptr() as *mut winapi::ctypes::c_void, buffer.len() as u32,
                                                    FileFullDirectoryInformation, 0, std::ptr::null_mut(), 0)};
            if result < 0 {
                break;
            }

            loop {
                let (head, body, _tail) = unsafe { buffer[offset..].align_to::<FILE_FULL_DIR_INFORMATION>() };
                let file_info = &body[0];
                let name_offset = name_member_offset + offset;
                offset += file_info.NextEntryOffset as usize;
                if file_info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
                    let mtime = unsafe { *file_info.LastWriteTime.QuadPart() as u32 };
                    let size = unsafe { *file_info.EndOfFile.QuadPart() as u32 };

                    let name_end = name_offset + file_info.FileNameLength as usize;
                    let name = String::from_utf8_lossy(&buffer[name_offset..name_end]).into_owned();
                    file_stats.insert(name, FileStat { mtime, size });
                }
                if file_info.NextEntryOffset  == 0 {
                    break;
                }
            }
        }
        // TODO look at making a wrapper object and use drop.
        unsafe {CloseHandle(handle);}
        file_stats

    }

    fn get_directory_handle(path: &Path) -> HANDLE {
        let name= CString::new(path.to_str().unwrap()).unwrap();
        let handle = unsafe {CreateFileA(name.as_ptr(), FILE_LIST_DIRECTORY, FILE_SHARE_WRITE | FILE_SHARE_READ | FILE_SHARE_DELETE, std::ptr::null_mut(), OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, std::ptr::null_mut())};
        handle
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