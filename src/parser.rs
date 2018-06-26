//! Stuff for parsing mtree files.
use util::{FromHex, FromDec};
use std::fmt;

/// An mtree file is a sequence of lines, each a semantic unit.
#[derive(Debug)]
pub enum MTreeLine<'a> {
    /// Blank lines are ignored.
    Blank,
    /// Lines starting with a '#' are ignored.
    Comment(&'a [u8]),
    /// Special commands (starting with '/') alter the behavior of later entries.
    Special(SpecialKind, Vec<Keyword<'a>>),
    /// If the first word does not contain a '/', it is a file in the current
    /// directory.
    Relative(&'a [u8], Vec<Keyword<'a>>),
    /// Change the current directory to the parent of the current directory.
    DotDot,
    /// If the first word does contain a '/', it is a file relative to the starting
    /// (not current) directory.
    Full(&'a [u8], Vec<Keyword<'a>>),
}

impl<'a> MTreeLine<'a> {
    pub fn from_bytes(input: &'a [u8]) -> Option<MTreeLine<'a>> {
        let mut parts = input.split(|ch| *ch == b' ')
            .filter(|word| ! word.is_empty());
        // Blank
        let first = match parts.next() {
            Some(f) => f,
            None => return Some(MTreeLine::Blank),
        };
        // Comment
        if first[0] == b'#' {
            return Some(MTreeLine::Comment(input));
        }
        // DotDot
        if first == b".." {
            return Some(MTreeLine::DotDot);
        }
        // the rest need params
        let mut params = Vec::new();
        for part in parts {
            let keyword = Keyword::from_bytes(part);
            debug_assert!(keyword.is_some(), 
                          "could not parse bytes: {}",
                          String::from_utf8_lossy(part));
            if let Some(keyword) = keyword {
                params.push(keyword);
            }
        }

        // Special
        if first[0] == b'/' {
            let kind = SpecialKind::from_bytes(&first[1..])?;
            Some(MTreeLine::Special(kind, params))
        // Full
        } else if first.contains(&b'/') {
            Some(MTreeLine::Full(first, params))
        } else {
            Some(MTreeLine::Relative(first, params))
        }
    }
}

/// A command that alters the behavior of later commands.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpecialKind {
    /// Set a default for future lines.
    Set,
    /// Unset a default for future lines.
    Unset,
}

impl SpecialKind {
    fn from_bytes(input: &[u8]) -> Option<SpecialKind> {
        Some(match input {
            b"set" => SpecialKind::Set,
            b"unset" => SpecialKind::Unset,
            _ => return None,
        })
    }
}

/// Each entry may have one or more key word
pub enum Keyword<'a> {
    /// `cksum` The checksum of the file using the default algorithm specified by
    /// the cksum(1) utility.
    Checksum(&'a [u8]),
    /// `device` The device number for *block* or *char* file types.
    Device(Device<'a>),
    /// `contents` The full pathname of a file that holds the contents of this file.
    Contents(&'a [u8]),
    /// `flags` The file flags as a symbolic name.
    Flags(&'a [u8]),
    /// `gid` The file group as a numeric value.
    Gid(u64),
    /// `gname` The file group as a symbolic name.
    Gname(&'a [u8]),
    /// `ignore` Ignore any file hierarchy below this line.
    Ignore,
    /// `inode` The inode number.
    Inode(u64),
    /// `link` The target of the symbolic link when type=link.
    Link(&'a [u8]),
    /// `md5|md5digest` The MD5 message digest of the file.
    Md5([u8; 16]),
    /// `mode` The current file's permissions as a numeric (octal) or symbolic value.
    Mode(&'a [u8]),
    /// `nlink` The number of hard links the file is expected to have.
    NLink(u64),
    /// `nochange` Make sure this file or directory exists but otherwise ignore 
    /// all attributes.
    NoChange,
    /// `optional` The file is optional; do not complain about the file if it is 
    /// not in the file hierarchy.
    Optional,
    /// `resdevice` The "resident" device number of the file, e.g. the ID of the
    /// device that contains the file. Its format is the same as the one for 
    /// `device`.
    ResidentDevice(Device<'a>),
    /// `rmd160|rmd160digest|ripemd160digest` The RIPEMD160 message digest of 
    /// the file.
    Rmd160([u8; 20]),
    /// `sha1|sha1digest` The FIPS 160-1 ("SHA-1") message digest of the file.
    Sha1([u8; 20]),
    /// `sha256|sha256digest` The FIPS 180-2 ("SHA-256") message digest of the file.
    Sha256([u8; 32]),
    /// `sha384|sha384digest` The FIPS 180-2 ("SHA-384") message digest of the file.
    Sha384([u8; 48]),
    /// `sha512|sha512digest` The FIPS 180-2 ("SHA-512") message digest of the file.
    Sha512([u8; 64]),
    /// `size` The size, in bytes, of the file.
    Size(u64),
    /// `time` The last modification time of the file
    Time(&'a [u8]),
    /// `type` The type of the file.
    Type(Type),
    /// The file owner as a numeric value.
    Uid(u64),
    /// The file owner as a symbolic name.
    Uname(&'a [u8]),
}

impl<'a> fmt::Debug for Keyword<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keyword::Checksum(ref inner) => 
                f.debug_tuple("Keyword::Checksum").field(inner).finish(),
            Keyword::Device(ref inner) =>
                f.debug_tuple("Keyword::Device").field(inner).finish(),
            Keyword::Contents(ref inner) =>
                f.debug_tuple("Keyword::Contents").field(inner).finish(),
            Keyword::Flags(ref inner) =>
                f.debug_tuple("Keyword::Flags").field(inner).finish(),
            Keyword::Gid(inner) =>
                f.debug_tuple("Keyword::Device").field(inner).finish(),
            Keyword::Gname(ref inner) =>
                f.debug_tuple("Keyword::Gname").field(inner).finish(),
            Keyword::Ignore =>
                f.write_str("Keyword::Ignore"),
            Keyword::Inode(inner) =>
                f.debug_tuple("Keyword::Inode").field(inner).finish(),
            Keyword::Link(ref inner) =>
                f.debug_tuple("Keyword::Link").field(inner).finish(),
            Keyword::Md5(inner) => {
                f.write_str("Keyword::Md5(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            }
            Keyword::Mode(ref inner) =>
                f.debug_tuple("Keyword::Mode").field(inner).finish(),
            Keyword::NLink(inner) =>
                f.debug_tuple("Keyword::NLink").field(inner).finish(),
            Keyword::NoChange =>
                f.write_str("Keyword::NoChange"),
            Keyword::Optional =>
                f.write_str("Keyword::Optional"),
            Keyword::ResidentDevice(inner) =>
                f.debug_tuple("Keyword::ResidentDevice").field(inner).finish(),
            Keyword::Rmd160(inner) => {
                f.write_str("Keyword::Md5(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            },
            Keyword::Sha1(inner) => {
                f.write_str("Keyword::Sha1(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            },
            Keyword::Sha256(inner) => {
                f.write_str("Keyword::Sha256(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            },
            Keyword::Sha384(inner) => {
                f.write_str("Keyword::Sha384(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            },
            Keyword::Sha512(inner) => {
                f.write_str("Keyword::Sha512(")?;
                f.debug_list().entries(inner.iter()).finish()?;
                f.write_str(")")
            },
            Keyword::Size(inner) =>
                f.debug_tuple("Keyword::Size").field(inner).finish(),
            Keyword::Time(ref inner) =>
                f.debug_tuple("Keyword::Time").field(inner).finish(),
            Keyword::Type(inner) =>
                f.debug_tuple("Keyword::Type").field(inner).finish(),
            Keyword::Uid(inner) =>
                f.debug_tuple("Keyword::Uid").field(inner).finish(),
            Keyword::Uname(ref inner) =>
                f.debug_tuple("Keyword::Uname").field(inner).finish(),
        }
    }
}

impl<'a> Keyword<'a> {
    /// Parse a keyword with optional value.
    fn from_bytes(input: &'a [u8]) -> Option<Keyword<'a>> {
        let mut iter = input.splitn(2, |ch| *ch == b'=');
        let key = iter.next()?;
        Some(match key {
            b"cksum" => Keyword::Checksum(iter.next()?),
            b"device" => Keyword::Device(Device::from_bytes(iter.next()?)?),
            b"contents" => Keyword::Contents(iter.next()?),
            b"flags" => Keyword::Flags(iter.next()?),
            b"gid" => Keyword::Gid(u64::from_dec(iter.next()?)?),
            b"gname" => Keyword::Gname(iter.next()?),
            b"ignore" => Keyword::Ignore,
            b"inode" => Keyword::Inode(u64::from_dec(iter.next()?)?),
            b"link" => Keyword::Link(iter.next()?),
            b"md5" | b"md5digest" 
                => Keyword::Md5(<[u8; 16]>::from_hex(iter.next()?)?),
            b"mode" => Keyword::Mode(iter.next()?),
            b"nlink" => Keyword::NLink(u64::from_dec(iter.next()?)?),
            b"nochange" => Keyword::NoChange,
            b"optional" => Keyword::Optional,
            b"resdevice" => 
                Keyword::ResidentDevice(Device::from_bytes(iter.next()?)?),
            b"rmd160" | b"rmd160digest" | b"ripemd160digest" =>
                Keyword::Rmd160(<[u8; 20]>::from_hex(iter.next()?)?),
            b"sha1" | b"sha1digest" => 
            Keyword::Sha1(<[u8; 20]>::from_hex(iter.next()?)?),
            b"sha256" | b"sha256digest" => 
                Keyword::Sha256(<[u8; 32]>::from_hex(iter.next()?)?),
            b"sha384" | b"sha384digest" => 
                Keyword::Sha384(<[u8; 48]>::from_hex(iter.next()?)?),
            b"sha512" | b"sha512digest" => 
                Keyword::Sha512(<[u8; 64]>::from_hex(iter.next()?)?),
            b"size" => Keyword::Size(u64::from_dec(iter.next()?)?),
            b"time" => Keyword::Time(iter.next()?),
            b"type" => Keyword::Type(Type::from_bytes(iter.next()?)?),
            b"uid" => Keyword::Uid(u64::from_dec(iter.next()?)?),
            b"uname" => Keyword::Uname(iter.next()?),
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Device<'a> {
    /// The device format
    format: Format<'a>,
    /// The device major identifier
    major: &'a [u8],
    /// The device minor identifier
    minor: &'a [u8],
    /// The device subunit identifier, if applicable.
    subunit: Option<&'a [u8]>,
}

impl<'a> Device<'a> {
    fn from_bytes(input: &'a [u8]) -> Option<Device<'a>> {
        let mut iter = input.splitn(4, |ch| *ch == b',');
        let format = Format::from_bytes(iter.next()?);
        let major = iter.next()?;
        let minor = iter.next()?;
        let subunit = iter.next(); // optional, so no '?'
        Some(Device { format, major, minor, subunit })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Format<'a> {
    Native,
    Bsd386,
    Bsd4,
    BsdOs,
    FreeBsd,
    Hpux,
    Isc,
    Linux,
    NetBsd,
    Osf1,
    Sco,
    Solaris,
    SunOs,
    Svr3,
    Svr4,
    Ultrix,
    Other(&'a [u8]),
}

impl<'a> Format<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Format<'a> {
        match bytes {
            b"native" => Format::Native,
            b"386bsd" => Format::Bsd386,
            b"4bsd" => Format::Bsd4,
            b"bsdos" => Format::BsdOs,
            b"freebsd" => Format::FreeBsd,
            b"hpux" => Format::Hpux,
            b"isc" => Format::Isc,
            b"linux" => Format::Linux,
            b"netbsd" => Format::NetBsd,
            b"osf1" => Format::Osf1,
            b"sco" => Format::Sco,
            b"solaris" => Format::Solaris,
            b"sunos" => Format::SunOs,
            b"svr3" => Format::Svr3,
            b"svr4" => Format::Svr4,
            b"ultrix" => Format::Ultrix,
            ref other => Format::Other(other)
        }
    }
}

#[test]
fn test_format_from_butes() {
    for (input, res) in vec![
        (&b"native"[..], Format::Native),
        (&b"386bsd"[..], Format::Bsd386),
        (&b"4bsd"[..], Format::Bsd4),
        (&b"bsdos"[..], Format::BsdOs),
        (&b"freebsd"[..], Format::FreeBsd),
        (&b"hpux"[..], Format::Hpux),
        (&b"isc"[..], Format::Isc),
        (&b"linux"[..], Format::Linux),
        (&b"netbsd"[..], Format::NetBsd),
        (&b"osf1"[..], Format::Osf1),
        (&b"sco"[..], Format::Sco),
        (&b"solaris"[..], Format::Solaris),
        (&b"sunos"[..], Format::SunOs),
        (&b"svr3"[..], Format::Svr3),
        (&b"svr4"[..], Format::Svr4),
        (&b"ultrix"[..], Format::Ultrix),
        (&b"other"[..], Format::Other(b"other"))
    ] {
        assert_eq!(Format::from_bytes(&input[..]), res);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    BlockDevice,
    CharacterDevice,
    Directory,
    Fifo,
    File,
    SymbolicLink,
    Socket,
}

impl Type {
    fn from_bytes(input: &[u8]) -> Option<Type> {
        Some(match input {
            b"block" => Type::BlockDevice,
            b"char" => Type::CharacterDevice,
            b"dir" => Type::Directory,
            b"fifo" => Type::Fifo,
            b"file" => Type::File,
            b"link" => Type::SymbolicLink,
            b"socket" => Type::Socket,
            _ => return None,
        })
    }
}

#[test]
fn test_type_from_bytes() {
    for (input, res) in vec![
        (&b"block"[..], Type::BlockDevice),
        (&b"char"[..], Type::CharacterDevice),
        (&b"dir"[..], Type::Directory),
        (&b"fifo"[..], Type::Fifo),
        (&b"file"[..], Type::File),
        (&b"link"[..], Type::SymbolicLink),
        (&b"socket"[..], Type::Socket),
    ] {
        assert_eq!(Type::from_bytes(&input[..]), Some(res));
    }
    assert!(Type::from_bytes(&b"other"[..]).is_none());
}
