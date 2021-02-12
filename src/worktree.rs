/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */
/*
Some discussion on directory walking metrics.

Using the llvm-project form https://github.com/llvm/llvm-project.git at commit,
0f9f0a4046e11c2b4c130640f343e3b2b5db08c1
Some metrics:

- 0.340s git status
- 0.700s using the walkdir crate from rust,
  https://github.com/BurntSushi/walkdir.git this utilized the walkdir-list
  command with ``-c`` to only show the count.
- 0.910s using fd with the ``-I`` flag to not look at git ignore.
- 1.120s using fd.  This dumped result to a log file
- 1.228s using libgit2 status example
- 4.000s Utilizing tokio and walking a directory asynchronously by blindly copying
  https://stackoverflow.com/a/58825638/4866781

So how do we get to the speed of git status if walkdir takes almost twice as long as git status?
git status needs to also look at the file sha's.

- 0.614s ``fd > /dev/null``.  It looks like writing and or updating the output file is having
  significant performance issues.  Not 100% why it's faster than walkdir.
- 0.400s ``fd -I  > /dev/null``.
- 0.847s ``fd -j 1 -I > /dev/null``.  It looks like fd uses threads, by default it seems to favor 12
  when using one it's noticeably slower.
- 0.511s ``fd -j 2 -I > /dev/null``
- 0.396s ``fd -j 3 -I > /dev/null``
- 0.356s ``fd -j 4 -I > /dev/null``
- 0.362s ``fd -j 5 -I > /dev/null``
- 0.390s ``fd -j 6 -I > /dev/null``  From 6-12 it can get down to 0.390s but 12 will often
  hit 0.400s

Looking into fd(find) more it looks like it uses the [ignore](https://crates.io/crates/ignore)
crate, an example here, http://blog.vmchale.com/article/directory-traversals

Using ignore directly here with:

    use ignore::WalkBuilder;

    for result in WalkBuilder::new("./").hidden(false).build() {
        println!("{:?}", result);
    }

2.420s

Instead of printing just accumulating the entries.
2.325s

Trying to walk in parallel `WalkBuilder::new(path).build_parallel().visit(&mut builder);`
1.123s

Connecting up thread counts `WalkBuilder::new(path).threads(1).build_parallel().visit(&mut builder);`
Need to look, but guessing this is not ignoring at the visitor...
    Number of threads             time
        1                           1.921s
        2                           1.119s
        3                           0.860s
        4                           0.715s
        5                           0.636s
        6                           0.590s
        7                           0.562s
        8                           0.550s
        9                           0.535s
       10                           0.520s
       11                           0.505s
       12                           0.500s

In working on getting the parallelwalker hooked up ran across this https://users.rust-lang.org/t/feedback-on-crate-for-parallel-recursive-directory-walk/25001

}
 */

use ignore::{WalkBuilder, ParallelVisitorBuilder, ParallelVisitor, DirEntry, WalkState};
use std::path::Path;

#[derive(Debug)]
pub struct WorkTreeError {
    message: String,
}

impl From<ignore::Error> for WorkTreeError {
    fn from(err: ignore::Error) -> WorkTreeError {
        WorkTreeError {
            message: err.to_string(),
        }
    }
}
/// A worktree of a repo.
///
/// Some common git internal terms.
///
/// - `oid` - Object ID.  This is often the SHA of an item.  It could be a commit, file blob, tree,
///     etc.
#[derive(Debug)]
pub struct WorkTree {
    path: String,
    pub entries: Vec<Entry>,
}

/// Represents an index entry, i.e. a file or blob
#[derive(PartialEq, Eq, Debug)]
pub struct Entry {
    // The docs call this "object name"
    sha: [u8; 20],
    name: String,
}

struct DirVisitor {
    pub entries: Vec<Entry>,
}

impl ParallelVisitor for DirVisitor{
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        let dir_entry = entry.unwrap();
        let name = dir_entry.path().to_str().unwrap().to_string();
        // self.entries.push(Entry {sha: *b"00000000000000000000", name: entry.ok_or(WorkTreeError{message: "FAIL WHALE".to_string()})?.to_string()});
        self.entries.push(Entry {sha: *b"00000000000000000000", name });
        return WalkState::Continue
    }
}

pub struct DirBuilder {
    pub entries: Vec<Entry>,
    // pub visitors: Vec<Box<DirVisitor>>,
}

impl<'s> ParallelVisitorBuilder<'s> for DirBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        let visitor = Box::new(DirVisitor {entries: vec![]});
        visitor
    }
}
impl WorkTree {
    /// Returns the index for the git repo at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to a git repo.  This logic will _not_ search up parent directories for
    ///     a git repo
    pub fn new(path: &Path) -> Result<WorkTree, WorkTreeError> {
        let mut builder = DirBuilder{entries: vec![]};
        WalkBuilder::new(path).threads(12).build_parallel().visit(&mut builder);
        // for result in WalkBuilder::new(path).build() {
            // entries.push(Entry {sha: *b"00000000000000000000", name: result?.path().to_str().ok_or(WorkTreeError{message: "FAIL WHALE".to_string()})?.to_string()});
            // println!("{:?}", result);
        // }
        let work_tree = WorkTree {
            path: String::from(path.to_str().unwrap()),
            entries: builder.entries
        };
        Ok(work_tree)
    }
}
