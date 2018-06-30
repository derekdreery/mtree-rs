extern crate failure;
#[macro_use]
extern crate failure_derive;

use std::io::{self, Read, BufRead};
use std::path::PathBuf;
use std::env;
use std::str;

mod parser;
mod util;

pub use parser::{ParserError, MTreeLine, Format, Type};
use parser::Keyword;

pub struct MTree {
    inner: Box<Iterator<Item=io::Result<Vec<u8>>>>,
    cwd: Option<PathBuf>,
    set_params: Params,
}

impl MTree {
    pub fn from_reader(reader: impl Read + 'static) -> MTree {
        let reader = io::BufReader::new(reader);
        MTree {
            inner: Box::new(reader.split(b'\n')
                // remove trailing '\r'
                .map(|line|
                    line.map(|mut line| {
                        if ! line.is_empty() && line[line.len()-1] == b'r' {
                            line.pop();
                        }
                        line
                    })
                )
            ),
            cwd: env::current_dir().ok(),
            set_params: Params::default(),
        }
    }
}

impl Iterator for MTree {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<io::Result<Entry>> {
        let line = match self.inner.next()? {
            Ok(line) => line,
            Err(e) => return Some(Err(e))
        };
        None
    }
}

pub struct Entry {
    pub path: PathBuf,
    pub params: Params,
}

#[derive(Default, Clone)]
pub struct Params {
    /// `cksum` The checksum of the file using the default algorithm specified by
    /// the cksum(1) utility.
    pub checksum: Option<u64>,
    /// `device` The device number for *block* or *char* file types.
    pub device: Option<Device>,
    /// `contents` The full pathname of a file that holds the contents of this file.
    pub contents: Option<Vec<u8>>,
    /// `flags` The file flags as a symbolic name.
    pub flags: Option<Vec<u8>>,
    /// `gid` The file group as a numeric value.
    pub gid: Option<u64>,
    /// `gname` The file group as a symbolic name.
    pub gname: Option<String>,
    /// `ignore` Ignore any file hierarchy below this line.
    pub ignore: bool,
    /// `inode` The inode number.
    pub inode: Option<u64>,
    /// `link` The target of the symbolic link when type=link.
    pub link: Option<Vec<u8>>,
    /// `md5|md5digest` The MD5 message digest of the file.
    pub md5: Option<[u8; 16]>,
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
    pub sha384: Option<[u8; 48]>,
    /// `sha512|sha512digest` The FIPS 180-2 ("SHA-512") message digest of the file.
    pub sha512: Option<[u8; 64]>,
    /// `size` The size, in bytes, of the file.
    pub size: Option<u64>,
    /// `time` The last modification time of the file
    pub time: Option<Vec<u8>>,
    /// `type` The type of the file.
    pub file_type: Option<Type>,
    /// The file owner as a numeric value.
    pub uid: Option<u64>,
    /// The file owner as a symbolic name.
    pub uname: Option<Vec<u8>>,
}

impl Params {
    fn set(&mut self, keywords: impl Iterator<Item=Keyword<'static>>) {
        for keyword in keywords {
            match keyword {
                Keyword::Checksum(cksum) => self.checksum = Some(cksum),
                Keyword::DeviceRef(device) => self.device = Some(device.to_device()),
                Keyword::Contents(contents) => self.contents = Some(contents.to_owned()),
                Keyword::Flags(flags) => self.flags = Some(flags.to_owned()),
                Keyword::Gid(gid) => self.gid = Some(gid),
                // Should be utf8: see https://unix.stackexchange.com/questions/21013/does-character-ä-in-usernames-cause-bugs-in-linux-systems
                // Same for user
                Keyword::Gname(gname) => self.gname = Some(str::from_utf8(gname)
                                                           .expect("group name to be utf8")
                                                           .to_owned()),
                Keyword::Ignore => self.ignore = true,
                Keyword::Inode(inode) => self.inode = Some(inode),
                Keyword::Link(link) => self.link = Some(link.to_owned()),
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
                Keyword::Time(time) => self.time = Some(time.to_owned()),
                Keyword::Type(ty) => self.file_type = Some(ty),
                Keyword::Uid(uid) => self.uid = Some(uid),
                Keyword::Uname(uname) => self.uname = Some(uname.to_owned()),
            }
        }
    }

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

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}

