use std::io::{self, Read, BufRead};
use std::iter;
use std::path::PathBuf;
use std::env;

mod parser;
mod util;

pub use parser::MTreeLine;

pub struct MTree {
    inner: Box<Iterator<Item=Result<Vec<u8>, io::Error>>>,
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
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        unimplemented!()
    }
}

pub struct Entry {
    path: PathBuf,
    params: Params,
}

#[derive(Default)]
pub struct Params {
    /// `cksum` The checksum of the file using the default algorithm specified by
    /// the cksum(1) utility.
    checksum: Option<Vec<u8>>,
    /// `device` The device number for *block* or *char* file types.
    device: Option<Device>,
    /// `contents` The full pathname of a file that holds the contents of this file.
    contenxt: Option<Vec<u8>>,
    /// `flags` The file flags as a symbolic name.
    flags: Option<Vec<u8>>,
    /// `gid` The file group as a numeric value.
    gid: Option<u64>,
    /// `gname` The file group as a symbolic name.
    gname Option<String>,
    /// `ignore` Ignore any file hierarchy below this line.
    ignore: bool,
    /// `inode` The inode number.
    inode: Option<u64>,
    /// `link` The target of the symbolic link when type=link.
    link: Option<Vec<u8>>,
    /// `md5|md5digest` The MD5 message digest of the file.
    md5: Option<[u8; 16]>,
    /// `mode` The current file's permissions as a numeric (octal) or symbolic value.
    mode: Option<Vec<u8>>,
    /// `nlink` The number of hard links the file is expected to have.
    nlink: Option<u64>,
    /// `nochange` Make sure this file or directory exists but otherwise ignore 
    /// all attributes.
    no_change: bool,
    /// `optional` The file is optional; do not complain about the file if it is 
    /// not in the file hierarchy.
    optional: bool,
    /// `resdevice` The "resident" device number of the file, e.g. the ID of the
    /// device that contains the file. Its format is the same as the one for 
    /// `device`.
    resident_device: Option<Device>,
    /// `rmd160|rmd160digest|ripemd160digest` The RIPEMD160 message digest of 
    /// the file.
    rmd160: Option<[u8; 20]>,
    /// `sha1|sha1digest` The FIPS 160-1 ("SHA-1") message digest of the file.
    sha1: Option<[u8; 20]>,
    /// `sha256|sha256digest` The FIPS 180-2 ("SHA-256") message digest of the file.
    sha256: Option<[u8; 32]>,
    /// `sha384|sha384digest` The FIPS 180-2 ("SHA-384") message digest of the file.
    sha384: Option<[u8; 48]>,
    /// `sha512|sha512digest` The FIPS 180-2 ("SHA-512") message digest of the file.
    sha512: Option<[u8; 64]>,
    /// `size` The size, in bytes, of the file.
    size: Option<u64>,
    /// `time` The last modification time of the file
    time: Option<Vec<u8>>,
    /// `type` The type of the file.
    type_: Option<Type>,
    /// The file owner as a numeric value.
    uid: Option<u64>,
    /// The file owner as a symbolic name.
    uname: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Device<'a> {
    /// The device format
    format: Format,
    /// The device major identifier
    major: Vec<u8>,
    /// The device minor identifier
    minor: Vec<u8>,
    /// The device subunit identifier, if applicable.
    subunit: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

