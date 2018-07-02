//! A library for iterating through entries of an mtree.
//!
//! *mtree* is a data format used for describing a sequence of files. Their location is record,
//! along with optional extra values like checksums, size, permissions etc.
//!
//! For details on the spec see [mtree(5)].
//!
//! # Examples
//!
//! ```
//! use mtree::MTree;
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! // We're going to load data from a string so this example with pass doctest,
//! // but there's no reason you can't use a file, or any other data source.
//! let raw_data = "
//! /set type=file uid=0 gid=0 mode=644
//! ./.BUILDINFO time=1523250074.300237174 size=8602 md5digest=13c0a46c2fb9f18a1a237d4904b6916e \
//!     sha256digest=db1941d00645bfaab04dd3898ee8b8484874f4880bf03f717adf43a9f30d9b8c
//! ./.PKGINFO time=1523250074.276237110 size=682 md5digest=fdb9ac9040f2e78f3561f27e5b31c815 \
//!     sha256digest=5d41b48b74d490b7912bdcef6cf7344322c52024c0a06975b64c3ca0b4c452d1
//! /set mode=755
//! ./usr time=1523250049.905171912 type=dir
//! ./usr/bin time=1523250065.373213293 type=dir
//! ";
//! let entries = MTree::from_reader(raw_data.as_bytes());
//! for entry in entries {
//!     // Normally you'd want to handle any errors
//!     let entry = entry.unwrap();
//!     // We can print out a human-readable copy of the entry
//!     println!("{}", entry);
//!     // Let's check that if there is a creation time, it's in the past
//!     if let Some(time) = entry.params.time {
//!         assert!(time < SystemTime::now());
//!     }
//!     // We might also want to take a checksum of the file, and compare it to the digests
//!     // supplied by mtree, but this example doesn't have access to a filesystem.
//! }
//! ```
//!
//! [mtree(5)]: https://www.freebsd.org/cgi/man.cgi?mtree(5)

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate smallvec;
#[macro_use]
extern crate newtype_array;
#[macro_use]
extern crate bitflags;

use std::io::{self, Read, BufRead, BufReader, Split};
use std::path::{Path, PathBuf};
use std::env;
use std::fmt;
use std::time::{UNIX_EPOCH, SystemTime};
use std::os::unix::ffi::OsStrExt;
use std::ffi::OsStr;
use smallvec::SmallVec;

mod parser;
mod util;

pub use parser::{ParserError, MTreeLine, Format, Type, FileMode, Perms};
use parser::{Keyword, SpecialKind};
pub use util::{Array48, Array64};

#[cfg(not(unix))]
compiler_error!("This library currently only supports unix, due to windows using utf-16 for paths");

/// An mtree parser (start here).
///
/// This is the main struct for the lib. Semantically, an mtree file is a sequence of filesystem
/// records. These are provided as an iterator. Use the `from_reader` function to construct an
/// instance.
pub struct MTree<R> where R: Read {
    /// The iterator over lines (lines are guaranteed to end in \n since we only support unix).
    inner: Split<BufReader<R>>,
    /// The current working directory for dir calculations.
    cwd: PathBuf,
    /// These are set with the '/set' and '/unset' special functions.
    default_params: Params,
}

impl<R> MTree<R> where R: Read {
    /// The constructor function for an MTree instance.
    pub fn from_reader(reader: R) -> MTree<R> {
        MTree {
            inner: BufReader::new(reader).split(b'\n'),
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            default_params: Params::default(),
        }
    }

    /// This is a helper function to make error handling easier.
    fn next_entry(&mut self, line: io::Result<Vec<u8>>) -> Result<Option<Entry>, Error> {
        let line = line?;
        let line = MTreeLine::from_bytes(&line)?;
        Ok(match line {
            MTreeLine::Blank | MTreeLine::Comment(_) => None,
            MTreeLine::Special(SpecialKind::Set, keywords) => {
                self.default_params.set_list(keywords.into_iter());
                None
            },
            // this won't work because keywords need to be parsed without arguments.
            MTreeLine::Special(SpecialKind::Unset, _keywords) => unimplemented!(),
            MTreeLine::Relative(path, keywords) => {
                let mut params = self.default_params.clone();
                params.set_list(keywords.into_iter());
                if self.cwd.file_name().is_none() {
                    panic!("relative without a current working dir");
                }
                Some(Entry {
                    path: self.cwd.join(OsStr::from_bytes(path)),
                    params,
                })
            },
            MTreeLine::DotDot => {
                self.cwd.pop();
                None
            },
            MTreeLine::Full(path, keywords) => {
                let mut params = self.default_params.clone();
                params.set_list(keywords.into_iter());
                Some(Entry {
                    path: Path::new(OsStr::from_bytes(path)).to_owned(),
                    params,
                })
            },
        })
    }
}

impl<R> Iterator for MTree<R> where R: Read {
    type Item = Result<Entry, Error>;

    fn next(&mut self) -> Option<Result<Entry, Error>> {
        while let Some(line) = self.inner.next() {
            match self.next_entry(line) {
                Ok(Some(entry)) => return Some(Ok(entry)),
                Ok(None) => (),
                Err(e) => return Some(Err(e))
            }
        }
        None
    }
}

/// An entry in the mtree file.
///
/// Entries have a path to the entity in question, and a list of optional params.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Entry {
    /// The path of this entry
    pub path: PathBuf,
    /// All parameters applicable to this entry
    pub params: Params,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, r#"mtree entry for "{}""#, self.path.display())?;
        write!(f, "{}", self.params)
    }
}

/// All possible parameters to an entry.
///
/// All parameters are optional. `ignore`, `nochange` and `optional` all have no value, and so
/// `true` represets their presence.
#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Params {
    /// `cksum` The checksum of the file using the default algorithm specified by
    /// the cksum(1) utility.
    pub checksum: Option<u64>,
    /// `device` The device number for *block* or *char* file types.
    pub device: Option<Device>,
    /// `contents` The full pathname of a file that holds the contents of this file.
    pub contents: Option<PathBuf>,
    /// `flags` The file flags as a symbolic name.
    pub flags: Option<Vec<u8>>,
    /// `gid` The file group as a numeric value.
    pub gid: Option<u64>,
    /// `gname` The file group as a symbolic name.
    ///
    /// The name can be up to 32 chars and must match regex `[a-z_][a-z0-9_-]*[$]?`.
    pub gname: Option<SmallVec<[u8; 32]>>,
    /// `ignore` Ignore any file hierarchy below this line.
    pub ignore: bool,
    /// `inode` The inode number.
    pub inode: Option<u64>,
    /// `link` The target of the symbolic link when type=link.
    pub link: Option<PathBuf>,
    /// `md5|md5digest` The MD5 message digest of the file.
    pub md5: Option<u128>,
    /// `mode` The current file's permissions as a numeric (octal) or symbolic value.
    pub mode: Option<FileMode>,
    /// `nlink` The number of hard links the file is expected to have.
    pub nlink: Option<u64>,
    /// `nochange` Make sure this file or directory exists but otherwise ignore
    /// all attributes.
    pub no_change: bool,
    /// `optional` The file is optional; do not complain about the file if it is
    /// not in the file hierarchy.
    pub optional: bool,
    /// `resdevice` The "resident" device number of the file, e.g. the ID of the
    /// device that contains the file. Its format is the same as the one for
    /// `device`.
    pub resident_device: Option<Device>,
    /// `rmd160|rmd160digest|ripemd160digest` The RIPEMD160 message digest of
    /// the file.
    pub rmd160: Option<[u8; 20]>,
    /// `sha1|sha1digest` The FIPS 160-1 ("SHA-1") message digest of the file.
    pub sha1: Option<[u8; 20]>,
    /// `sha256|sha256digest` The FIPS 180-2 ("SHA-256") message digest of the file.
    pub sha256: Option<[u8; 32]>,
    /// `sha384|sha384digest` The FIPS 180-2 ("SHA-384") message digest of the file.
    pub sha384: Option<Array48<u8>>,
    /// `sha512|sha512digest` The FIPS 180-2 ("SHA-512") message digest of the file.
    pub sha512: Option<Array64<u8>>,
    /// `size` The size, in bytes, of the file.
    pub size: Option<u64>,
    /// `time` The last modification time of the file.
    pub time: Option<SystemTime>,
    /// `type` The type of the file.
    pub file_type: Option<Type>,
    /// The file owner as a numeric value.
    pub uid: Option<u64>,
    /// The file owner as a symbolic name.
    ///
    /// The name can be up to 32 chars and must match regex `[a-z_][a-z0-9_-]*[$]?`.
    pub uname: Option<SmallVec<[u8; 32]>>,
}

impl Params {

    fn set_list<'a>(&mut self, keywords: impl Iterator<Item=Keyword<'a>>) {
        for keyword in keywords {
            self.set(keyword);
        }
    }

    /// Set a parameter from a parsed keyword.
    fn set(&mut self, keyword: Keyword<'_>) {
        match keyword {
            Keyword::Checksum(cksum) => self.checksum = Some(cksum),
            Keyword::DeviceRef(device) => self.device = Some(device.to_device()),
            Keyword::Contents(contents) =>
                self.contents = Some(Path::new(OsStr::from_bytes(contents)).to_owned()),
            Keyword::Flags(flags) => self.flags = Some(flags.to_owned()),
            Keyword::Gid(gid) => self.gid = Some(gid),
            Keyword::Gname(gname) => self.gname = Some({
                let mut vec = SmallVec::new();
                vec.extend_from_slice(gname);
                vec
            }),
            Keyword::Ignore => self.ignore = true,
            Keyword::Inode(inode) => self.inode = Some(inode),
            Keyword::Link(link) => self.link = Some(Path::new(OsStr::from_bytes(link)).to_owned()),
            Keyword::Md5(md5) => self.md5 = Some(md5),
            Keyword::Mode(mode) => self.mode = Some(mode),
            Keyword::NLink(nlink) => self.nlink = Some(nlink),
            Keyword::NoChange => self.no_change = false,
            Keyword::Optional => self.optional = false,
            Keyword::ResidentDeviceRef(device) =>
                self.resident_device = Some(device.to_device()),
            Keyword::Rmd160(rmd160) => self.rmd160 = Some(rmd160),
            Keyword::Sha1(sha1) => self.sha1 = Some(sha1),
            Keyword::Sha256(sha256) => self.sha256 = Some(sha256),
            Keyword::Sha384(sha384) => self.sha384 = Some(sha384),
            Keyword::Sha512(sha512) => self.sha512 = Some(sha512),
            Keyword::Size(size) => self.size = Some(size),
            Keyword::Time(time) => self.time = Some(UNIX_EPOCH + time),
            Keyword::Type(ty) => self.file_type = Some(ty),
            Keyword::Uid(uid) => self.uid = Some(uid),
            Keyword::Uname(uname) => self.uname = Some({
                let mut vec = SmallVec::new();
                vec.extend_from_slice(uname);
                vec
            }),
        }
    }

    /*
    /// Empty this params list (better mem usage than creating a new one).
    fn clear(&mut self) {
        self.checksum = None;
        self.device = None;
        self.contents = None;
        self.flags = None;
        self.gid = None;
        self.gname = None;
        self.ignore = false;
        self.inode = None;
        self.link = None;
        self.md5 = None;
        self.mode = None;
        self.nlink = None;
        self.no_change = false;
        self.optional = false;
        self.resident_device = None;
        self.rmd160 = None;
        self.sha1 = None;
        self.sha256 = None;
        self.sha384 = None;
        self.sha512 = None;
        self.size = None;
        self.time = None;
        self.file_type = None;
        self.uid = None;
        self.uname = None;
    }
    */
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(v) = self.checksum {
            writeln!(f, "checksum: {}", v)?;
        }
        if let Some(ref v) = self.device {
            writeln!(f, "device: {:?}", v)?;
        }
        if let Some(ref v) = self.contents {
            writeln!(f, "contents: {}", v.display())?;
        }
        if let Some(ref v) = self.flags {
            writeln!(f, "flags: {:?}", v)?;
        }
        if let Some(v) = self.gid {
            if v != 0 {
                writeln!(f, "gid: {}", v)?;
            }
        }
        if let Some(ref v) = self.gname {
            writeln!(f, "gname: {}", String::from_utf8_lossy(v))?;
        }
        if self.ignore {
            writeln!(f, "ignore")?;
        }
        if let Some(v) = self.inode {
            writeln!(f, "inode: {}", v)?;
        }
        if let Some(ref v) = self.link {
            writeln!(f, "link: {}", v.display())?;
        }
        if let Some(ref v) = self.md5 {
            writeln!(f, "md5: {:x}", v)?;
        }
        if let Some(ref v) = self.mode {
            writeln!(f, "mode: {}", v)?;
        }
        if let Some(v) = self.nlink {
            writeln!(f, "nlink: {}", v)?;
        }
        if self.no_change {
            writeln!(f, "no change")?;
        }
        if self.optional {
            writeln!(f, "optional")?;
        }
        if let Some(ref v) = self.resident_device {
            writeln!(f, "resident device: {:?}", v)?;
        }
        if let Some(ref v) = self.rmd160 {
            write!(f, "rmd160: ")?;
            for ch in v {
                write!(f, "{:x}", ch)?;
            }
            writeln!(f)?;
        }
        if let Some(ref v) = self.sha1 {
            write!(f, "sha1: ")?;
            for ch in v {
                write!(f, "{:x}", ch)?;
            }
            writeln!(f)?;
        }
        if let Some(ref v) = self.sha256 {
            write!(f, "sha256: ")?;
            for ch in v {
                write!(f, "{:x}", ch)?;
            }
            writeln!(f)?;
        }
        if let Some(ref v) = self.sha384 {
            write!(f, "sha384: ")?;
            for ch in v {
                write!(f, "{:x}", ch)?;
            }
            writeln!(f)?;
        }
        if let Some(ref v) = self.sha512 {
            write!(f, "sha512: ")?;
            for ch in v {
                write!(f, "{:x}", ch)?;
            }
            writeln!(f)?;
        }
        if let Some(v) = self.size {
            writeln!(f, "size: {}", v)?;
        }
        if let Some(v) = self.time {
            writeln!(f, "creation time: {:?}", v)?;
        }
        if let Some(v) = self.file_type {
            writeln!(f, "file type: {}", v)?;
        }
        if let Some(v) = self.uid {
            if v != 0 {
                writeln!(f, "uid: {}", v)?;
            }
        }
        if let Some(ref v) = self.uname {
            writeln!(f, "uname: {}", String::from_utf8_lossy(v))?;
        }
        Ok(())
    }
}

/// A unix device.
///
/// The parsing for this could probably do with some work.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Device {
    /// The device format.
    pub format: Format,
    /// The device major identifier.
    pub major: Vec<u8>,
    /// The device minor identifier.
    pub minor: Vec<u8>,
    /// The device subunit identifier, if applicable.
    pub subunit: Option<Vec<u8>>,
}

/// The error type for this crate.
///
/// There are 2 possible ways that this lib can fail - there can be a problem parsing a record, or
/// there can be a fault in the underlying reader.
#[derive(Debug, Fail)]
pub enum Error {
    /// There was an i/o error reading data from the reader.
    #[fail(display="an i/o error occured while reading the mtree")]
    Io(#[cause] io::Error),
    /// There was a problem parsing the records.
    #[fail(display="an error occured while parsing the mtree")]
    Parser(#[cause] ParserError),
}

impl From<io::Error> for Error {
    fn from(from: io::Error) -> Error {
        Error::Io(from)
    }
}

impl From<parser::ParserError> for Error {
    fn from(from: parser::ParserError) -> Error {
        Error::Parser(from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}

