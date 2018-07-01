extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate smallvec;
#[macro_use]
extern crate newtype_array;

use std::io::{self, Read, BufRead, BufReader, Split};
use std::path::{Path, PathBuf};
use std::env;
use std::time::Duration;
use std::os::unix::ffi::OsStrExt;
use std::ffi::OsStr;
use smallvec::SmallVec;

mod parser;
mod util;

pub use parser::{ParserError, MTreeLine, Format, Type};
use parser::{Keyword, SpecialKind};
pub use util::{Array48, Array64};

#[cfg(not(unix))]
compiler_error!("This library currently only supports unix, due to windows using utf-16 for paths");

pub struct MTree<R> where R: Read {
    /// the iterator over lines (lines are guaranteed to end in \n since we only support unix)
    inner: Split<BufReader<R>>,
    /// The current working directory for dir calculations.
    cwd: PathBuf,
    /// These are set with the '/set' and '/unset' special functions
    default_params: Params,
}

impl<R> MTree<R> where R: Read {
    /// The constructor function for an MTree instance
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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Entry {
    pub path: PathBuf,
    pub params: Params,
}

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
    pub mode: Option<Vec<u8>>,
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
    /// `time` The last modification time of the file
    pub time: Option<Duration>,
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
            Keyword::Mode(mode) => self.mode = Some(mode.to_owned()),
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
            Keyword::Time(time) => self.time = Some(time),
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

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Device {
    /// The device format
    format: Format,
    /// The device major identifier
    major: Vec<u8>,
    /// The device minor identifier
    minor: Vec<u8>,
    /// The device subunit identifier, if applicable.
    subunit: Option<Vec<u8>>,
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display="an i/o error occured while reading the mtree")]
    Io(#[cause] io::Error),
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

