/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use nom::bytes::complete::tag;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::sequence::tuple;
use nom::take;

use nom::do_parse;
use nom::IResult;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use crate::direntry::{DirEntry, FileStat};

use std::collections::HashMap;

#[derive(Debug)]
pub struct GitStatusError {
    message: String,
}

impl From<io::Error> for GitStatusError {
    fn from(err: io::Error) -> GitStatusError {
        GitStatusError {
            message: err.to_string(),
        }
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for GitStatusError {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> GitStatusError {
        GitStatusError {
            message: err.to_string(),
        }
    }
}

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
#[derive(Debug, Default)]
pub struct Index {
    path: String,
    oid: [u8; 20],
    header: Header,
    pub entries: HashMap<String, Vec<DirEntry>>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
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
    pub fn new(path: &Path) -> Result<Index, GitStatusError> {
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

    /// Returns the oid(Object ID) for the index.
    ///
    /// The object ID of an index is the object ID of the tree which the index represents.
    pub fn oid(&self) -> &[u8] {
        &self.oid
    }

    /// Reads in the header from the provided stream
    ///
    ///
    fn read_header(stream: &[u8]) -> IResult<&[u8], Header> {
        let signature = tag("DIRC");

        let (input, (_, version, entries)) = tuple((signature, be_u32, be_u32))(stream)?;

        Ok((input, Header { version, entries }))
    }

    /// Reads in entry from the provided stream
    ///
    ///
    fn read_entry(stream: &[u8]) -> IResult<&[u8], (String, DirEntry)> {
        let (output, (mtime, size, sha, full_name)) = do_parse!(
            stream,
            take!(8)
                >> mtime: be_u32
                >> take!(24)
                >> size: be_u32
                >> sha: take!(20)
                >> name_size: be_u16
                >> name: take!(name_size)
                >> take!(8 - ((62 + name_size) % 8))
                >> (mtime, size, sha, String::from_utf8(name.to_vec()).unwrap())
        )?;

        let full_path = Path::new(&full_name);
        let parent_path = full_path.parent().unwrap().to_str().unwrap();
        let name = full_path.file_name().unwrap().to_str().unwrap().to_string();
        let entry = DirEntry {
            stat: FileStat { size, mtime },
            sha: sha.try_into().unwrap(),
            name,
        };
        Ok((output, (parent_path.to_string(), entry)))
    }

    // Get the directory entry and populate any parent entries that don't exist
    fn get_directory_entry<'a>(
        name: &str,
        directory_map: &'a mut HashMap<String, Vec<DirEntry>>,
    ) -> &'a mut Vec<DirEntry> {
        let _entry = directory_map.get(name);
        let directory_entry = match _entry {
            Some(_entry) => directory_map.get_mut(name).unwrap(),
            None => {
                for ancestor in Path::new(name).ancestors() {
                    directory_map
                        .entry(ancestor.to_str().unwrap().to_string())
                        .or_insert_with(Vec::<DirEntry>::new);
                }
                directory_map.get_mut(name).unwrap()
            }
        };

        directory_entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_header_version_2() {
        let version: u32 = 2;
        let entries: u32 = 3;
        let mut header: Vec<u8> = vec![];
        header.extend(b"DIRC");
        header.extend(&version.to_be_bytes());
        header.extend(&entries.to_be_bytes());
        assert_eq!(
            Index::read_header(&header),
            Ok((&b""[..], Header { version, entries }))
        );
    }

    #[test]
    fn test_read_header_version_3() {
        let version: u32 = 3;
        let entries: u32 = 9;
        let mut header: Vec<u8> = vec![];
        header.extend(b"DIRC");
        header.extend(&version.to_be_bytes());
        header.extend(&entries.to_be_bytes());
        assert_eq!(
            Index::read_header(&header),
            Ok((&b""[..], Header { version, entries }))
        );
    }

    #[test]
    fn test_read_header_leaves_subsequent_bytes_in_stream() {
        let version: u32 = 4;
        let entries: u32 = 2;
        let mut header: Vec<u8> = vec![];
        header.extend(b"DIRC");
        header.extend(&version.to_be_bytes());
        header.extend(&entries.to_be_bytes());
        header.extend(b"tail stuff");
        assert_eq!(
            Index::read_header(&header),
            Ok((&b"tail stuff"[..], Header { version, entries }))
        );
    }

    #[test]
    fn test_read_of_file_entry() {
        let name = b"some/file/name";
        let sha = b"abacadaba2376182368a";
        let mut stream: Vec<u8> = vec![];
        let ctime: u64 = 10;
        stream.extend(&ctime.to_be_bytes());
        let mtime_s: u32 = 20;
        stream.extend(&mtime_s.to_be_bytes());
        let mtime_ns: u32 = 25;
        stream.extend(&mtime_ns.to_be_bytes());
        let dev: u32 = 30;
        stream.extend(&dev.to_be_bytes());
        let ino: u32 = 30;
        stream.extend(&ino.to_be_bytes());
        let mode: u32 = 40;
        stream.extend(&mode.to_be_bytes());
        let uid: u32 = 50;
        stream.extend(&uid.to_be_bytes());
        let gid: u32 = 60;
        stream.extend(&gid.to_be_bytes());
        let file_size: u32 = 70;
        stream.extend(&file_size.to_be_bytes());
        stream.extend(sha);
        let name_length: u16 = name.len() as u16;
        stream.extend(&name_length.to_be_bytes());
        stream.extend(name);
        let pad_length = 8 - ((62 + name_length) % 8);
        stream.extend(vec![0; pad_length as usize]);
        assert_eq!(
            Index::read_entry(&stream),
            Ok((
                &b""[..],
                (
                    "some/file".to_string(),
                    DirEntry {
                        stat: FileStat {
                            mtime: 20,
                            size: 70,
                        },
                        sha: *sha,
                        name: "name".to_string()
                    }
                )
            ))
        );
    }

    #[test]
    fn test_read_entry_new_name_irrelevant_prefix() {
        let name = b"a/different/name/to/a/file/with.ext";
        let sha = b"ab7ca9aba237a18e3f8a";
        let mut stream: Vec<u8> = vec![0; 40];
        stream.extend(sha);
        let name_length: u16 = name.len() as u16;
        stream.extend(&name_length.to_be_bytes());
        stream.extend(name);
        let pad_length = 8 - ((62 + name_length) % 8);
        stream.extend(vec![0; pad_length as usize]);
        assert_eq!(
            Index::read_entry(&stream),
            Ok((
                &b""[..],
                (
                    "a/different/name/to/a/file".to_string(),
                    DirEntry {
                        stat: FileStat { mtime: 0, size: 0 },
                        sha: *sha,
                        name: "with.ext".to_string()
                    }
                )
            ))
        );
    }

    #[test]
    fn test_read_of_file_entry_leaves_remainder() {
        let name = b"a/file";
        let sha = b"ab7ca9aba237a18e3f8a";
        let mut stream: Vec<u8> = vec![0; 40];
        stream.extend(sha);
        let name_length: u16 = name.len() as u16;
        stream.extend(&name_length.to_be_bytes());
        stream.extend(name);
        let pad_length = 8 - ((62 + name_length) % 8);
        stream.extend(vec![0; pad_length as usize]);
        let suffix = b"what";
        stream.extend(suffix);
        let read = Index::read_entry(&stream);
        assert_eq!(
            read,
            Ok((
                &suffix[..],
                (
                    "a".to_string(),
                    DirEntry {
                        stat: FileStat { mtime: 0, size: 0 },
                        sha: *sha,
                        name: "file".to_string()
                    }
                )
            ))
        );
    }

    #[test]
    fn test_read_of_file_entry_leaves_remainder_when_no_pad_needed() {
        let name = b"niners999";
        let sha = b"ab7ca9aba437ae8e3f8a";
        let mut stream: Vec<u8> = vec![0; 40];
        stream.extend(sha);
        let name_length: u16 = name.len() as u16;
        stream.extend(&name_length.to_be_bytes());
        stream.extend(name);
        let pad_length = 1;
        stream.extend(vec![0; pad_length as usize]);
        let suffix = b"sure";
        stream.extend(suffix);
        let read = Index::read_entry(&stream);
        assert_eq!(
            read,
            Ok((
                &suffix[..],
                (
                    "".to_string(),
                    DirEntry {
                        stat: FileStat { mtime: 0, size: 0 },
                        sha: *sha,
                        name: "niners999".to_string()
                    }
                )
            ))
        );
    }

    #[test]
    fn test_read_of_file_entry_leaves_remainder_when_full_pad_needed() {
        let name = b"22";
        let sha = b"ab7ca9aba437ae8e3f8a";
        let mut stream: Vec<u8> = vec![0; 40];
        stream.extend(sha);
        let name_length: u16 = name.len() as u16;
        stream.extend(&name_length.to_be_bytes());
        stream.extend(name);
        let pad_length = 8;
        stream.extend(vec![0; pad_length as usize]);
        let suffix = b"Iknow";
        stream.extend(suffix);
        let read = Index::read_entry(&stream);
        assert_eq!(
            read,
            Ok((
                &suffix[..],
                (
                    "".to_string(),
                    DirEntry {
                        stat: FileStat { mtime: 0, size: 0 },
                        sha: *sha,
                        name: "22".to_string()
                    }
                )
            ))
        );
    }

    #[test]
    fn test_get_directory_entry_at_root() {
        let rooted_dir = "";
        let mut map = HashMap::new();
        Index::get_directory_entry(rooted_dir, &mut map);
        assert_eq!(true, map.len() == 1 && map.contains_key(""));
    }

    #[test]
    fn test_get_directory_3_levels_deep() {
        let deep_dir = "1/2/3";
        let mut map = HashMap::new();
        Index::get_directory_entry(deep_dir, &mut map);

        let directories = vec!["", "1", "1/2", "1/2/3"];
        assert_eq!(
            true,
            map.len() == directories.len() && directories.iter().all(|d| map.contains_key(*d))
        );
    }
}
