/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use std::path::Path;
use std::collections::HashMap;

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
        let mut file_stats = HashMap::new();
        let stat = FileStat{mtime: 0, size: 0};
        file_stats.insert("what".to_string(), stat);


        let dirstat = DirectoryStat{directory: path.to_str().unwrap().to_string(), file_stats};
        dirstat
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
    fn test_load_dir_stat() {

        let names = vec!["one"];
        let files = names.iter().map(|n| Path::new(n)).collect();
        let temp_dir = temp_tree(files);

        let dirstat = DirectoryStat::new(&temp_dir);
        assert_eq!(
            dirstat.file_stats.len(),
            1
        );
    }
}