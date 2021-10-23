/*
 *          Copyright Nick G. 2021.
 * Distributed under the Boost Software License, Version 1.0.
 *    (See accompanying file LICENSE or copy at
 *          https://www.boost.org/LICENSE_1_0.txt)
 */

use nom::bits;
use nom::bytes::complete::tag;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::sequence::tuple;
use nom::take;
use nom::take_bits;
use nom::tuple;

use nom::do_parse;
use nom::IResult;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use crate::direntry::{DirEntry, FileStat, ObjectType};

use crate::error::StatusError;
use std::collections::HashMap;

impl From<nom::Err<nom::error::Error<&[u8]>>> for StatusError {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> StatusError {
        StatusError {
            message: err.to_string(),
        }
    }
}

// A function for parsing the name size of an index entry.
// This assumes the input is at the 16 bit flags field.
//
//      A 16-bit 'flags' field split into (high to low bits)
//      - 1-bit assume-valid flag
//      - 1-bit extended flag (must be zero in version 2)
//      - 2-bit stage (during merge)
//      - 12-bit name length if the length is less than 0xFFF; otherwise 0xFFF is stored in this
//        field.
//
// Note: This currently throws away the `stage` entry which means this doesn't properly handle
//       merged files.
//
// To be honest, I'm not sure exactly why I wasn't able to do this in place next to the rest of
// the entry parsing, I think it has to do with treating the byte stream as bits.
// I think it's fairly reasonable that one needs to end the input at a byte boundary so anything
// that needs to be broken into bits should either be done after the fact of in a function like
// this where all the bits within a set of bytes can be processed.
//
// Also trying to put this as a function in the impl block for Index resulted in some compilation
// errors.  Not sure on why, my macro knowledge is next to nothing.
fn parse_name_size(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, b): (&[u8], (u8, u8, u16)) = do_parse!(
        input,
        b: bits!(tuple!(take_bits!(2u8), take_bits!(2u8), take_bits!(12u16))) >> (b)
    )?;
    // I tried to just return the u16 from the do_parse macro, but I kept hitting compiler errors
    // so I decided to fall back to full parse there and access the tuple entry here outside of the
    // do_parse
    Ok((input, b.2))
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
    pub fn new(path: &Path) -> Result<Index, StatusError> {
        let oid: [u8; 20] = [0; 20];
        let mut buffer: Vec<u8> = Vec::new();
        File::open(&path).and_then(|mut f| f.read_to_end(&mut buffer))?;
        let (mut contents, header) = Index::read_header(&buffer)?;
        let mut entries = HashMap::new();
        for _ in 0..header.entries {
            let (local_contents, (directory, entry)) = Index::read_entry(contents)?;
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
        let (output, (mtime_s, mtime_ns, mode, size, sha, full_name)) = do_parse!(
            stream,
            take!(8)
                >> mtime_s: be_u32
                >> mtime_ns: be_u32
                >> take!(10)
                >> mode: be_u16
                >> take!(8)
                >> size: be_u32
                >> sha: take!(20)
                >> name_size: parse_name_size
                >> name: take!(name_size)
                >> take!(8 - ((62 + name_size) % 8))
                >> (
                    mtime_s,
                    mtime_ns,
                    mode,
                    size,
                    sha,
                    String::from_utf8(name.to_vec()).unwrap()
                )
        )?;

        let object_bits = mode >> 12;
        let object_type = match object_bits {
            0b1110 => ObjectType::GitLink,
            0b1010 => ObjectType::SymLink,
            _ => ObjectType::Regular,
        };

        let full_path = Path::new(&full_name);
        let parent_path = full_path.parent().unwrap().to_str().unwrap();
        let name = full_path.file_name().unwrap().to_str().unwrap().to_string();
        // Git times are really a duration since unix Epoch
        let mtime = Duration::new(mtime_s.into(), mtime_ns).as_nanos();
        let entry = DirEntry {
            stat: FileStat { mtime, size },
            sha: sha.try_into().unwrap(),
            name,
            object_type,
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
    use std::fs;
    use temp_testdir::TempDir;

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
                            mtime: (20 * 1_000_000_000) + 25,
                            size: 70,
                        },
                        sha: *sha,
                        object_type: ObjectType::Regular,
                        name: "name".to_string(),
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
                        object_type: ObjectType::Regular,
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
                        object_type: ObjectType::Regular,
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
                        object_type: ObjectType::Regular,
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
                        object_type: ObjectType::Regular,
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

    #[test]
    fn test_merged_file() {
        let temp_dir = TempDir::default();
        let version: u32 = 2;
        let entries: u32 = 2;
        let name = b"some_file";
        let sha = b"abacadaba2376182368a";
        let mut stream: Vec<u8> = vec![];
        stream.extend(b"DIRC");
        stream.extend(&version.to_be_bytes());
        stream.extend(&entries.to_be_bytes());
        for entry in 0..entries {
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
            let mut name_length: u16 = name.len() as u16;
            //The different stage numbers are not really used during git-add command. They are used for handling merge conflicts. In a nutshell:
            //Slot 0: “normal”, un-conflicted, all-is-well entry.
            //Slot 1: “base”, the common ancestor version.
            //Slot 2: “ours”, the target (HEAD) version.
            //Slot 3: “theirs”, the being-merged-in version.
            let stage = match entry {
                0 => 0,
                _ => 0b0100000000000000,
            };
            name_length |= stage;
            stream.extend(&name_length.to_be_bytes());
            stream.extend(name);
            let pad_length = 8 - ((62 + name_length) % 8);
            stream.extend(vec![0; pad_length as usize]);
        }
        let index_file = temp_dir.join("some_index");
        fs::write(&index_file, stream).unwrap();
        let index = Index::new(&index_file).unwrap();
        let root = index.entries.get("").unwrap();

        assert_eq!(root.len(), 2);
    }
}
