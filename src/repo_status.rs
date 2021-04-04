/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use crate::{WorkTree, TreeDiff};

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct RepoStatus {
    work_tree_diff: WorkTree,
    index_diff: TreeDiff,
}

impl RepoStatus {
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<RepoStatus, StatusError> {
        let oid: [u8; 20] = [0; 20];
        let mut buffer: Vec<u8> = Vec::new();
        File::open(&path).and_then(|mut f| f.read_to_end(&mut buffer))?;
        let (mut contents, header) = Index::read_header(&buffer)?;
        let mut entries = HashMap::new();
        for _ in 0..header.entries {
            let (local_contents, (directory, entry)) = Index::read_entry(&contents)?;
            let directory_entry = Index::get_directory_entry(&directory, &mut entries);
            directory_entry.push(entry);
            contents = local_contents;
        }
        let index = Index {
            path: String::from(path.to_str().unwrap()),
            oid,
            header,
            entries,
        };
        Ok(index)
    }
