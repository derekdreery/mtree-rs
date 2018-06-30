//! Stuff for parsing mtree files.
use util::{FromHex, FromDec};
use std::fmt;

use super::Device;

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
    pub fn from_bytes(input: &'a [u8]) -> ParserResult<MTreeLine<'a>> {
        let mut parts = input.split(|ch| *ch == b' ')
            .filter(|word| ! word.is_empty());
        // Blank
        let first = match parts.next() {
            Some(f) => f,
            None => return Ok(MTreeLine::Blank),
        };
        // Comment
        if first[0] == b'#' {
            return Ok(MTreeLine::Comment(input));
        }
        // DotDot
        if first == b".." {
            return Ok(MTreeLine::DotDot);
        }
        // the rest need params
        let mut params = Vec::new();
        for part in parts {
            let keyword = Keyword::from_bytes(part);
            debug_assert!(keyword.is_ok(),
                          "could not parse bytes: {}",
                          String::from_utf8_lossy(part));
            if let Ok(keyword) = keyword {
                params.push(keyword);
            }
        }

        // Special
        if first[0] == b'/' {
            let kind = SpecialKind::from_bytes(&first[1..])?;
            Ok(MTreeLine::Special(kind, params))
        // Full
        } else if first.contains(&b'/') {
            Ok(MTreeLine::Full(first, params))
        } else {
            Ok(MTreeLine::Relative(first, params))
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
    fn from_bytes(input: &[u8]) -> ParserResult<SpecialKind> {
        Ok(match input {
            b"set" => SpecialKind::Set,
            b"unset" => SpecialKind::Unset,
            _ => return Err(format!(r#""{}" is not a special command"#,
                                    String::from_utf8_lossy(input)).into()),
        })
    }
}

/// Each entry may have one or more key word
pub enum Keyword<'a> {
    /// `cksum` The checksum of the file using the default algorithm specified by
    /// the cksum(1) utility.
    // I'm pretty sure u32 is big enough, but I'm using u64 because I'm not sure this is
    // guaranteed.
    Checksum(u64),
    /// `device` The device number for *block* or *char* file types.
    DeviceRef(DeviceRef<'a>),
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
    ResidentDeviceRef(DeviceRef<'a>),
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
            Keyword::DeviceRef(ref inner) =>
                f.debug_tuple("Keyword::DeviceRef").field(inner).finish(),
            Keyword::Contents(ref inner) =>
                f.debug_tuple("Keyword::Contents").field(inner).finish(),
            Keyword::Flags(ref inner) =>
                f.debug_tuple("Keyword::Flags").field(inner).finish(),
            Keyword::Gid(inner) =>
                f.debug_tuple("Keyword::DeviceRef").field(inner).finish(),
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
            Keyword::ResidentDeviceRef(inner) =>
                f.debug_tuple("Keyword::ResidentDeviceRef").field(inner).finish(),
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
    fn from_bytes(input: &'a [u8]) -> ParserResult<Keyword<'a>> {
        fn next<'a>(field: &'static str, val: Option<&'a [u8]>)
            -> ParserResult<&'a [u8]>
        {
            val.ok_or_else(|| format!(r#""{}" requires a parameter, none found"#, field).into())
        }
        let mut iter = input.splitn(2, |ch| *ch == b'=');
        let key = iter.next().unwrap(); // cannot fail
        Ok(match key {
            b"cksum" => Keyword::Checksum(u64::from_dec(next("cksum", iter.next())?)?),
            b"device" => Keyword::DeviceRef(DeviceRef::from_bytes(next("devices", iter.next())?)?),
            b"contents" => Keyword::Contents(next("contents", iter.next())?),
            b"flags" => Keyword::Flags(next("flags", iter.next())?),
            b"gid" => Keyword::Gid(u64::from_dec(next("gid", iter.next())?)?),
            b"gname" => Keyword::Gname(next("gname", iter.next())?),
            b"ignore" => Keyword::Ignore,
            b"inode" => Keyword::Inode(u64::from_dec(next("inode", iter.next())?)?),
            b"link" => Keyword::Link(next("link", iter.next())?),
            b"md5" | b"md5digest"
                => Keyword::Md5(<[u8; 16]>::from_hex(next("md5|md5digest", iter.next())?)?),
            b"mode" => Keyword::Mode(next("mode", iter.next())?),
            b"nlink" => Keyword::NLink(u64::from_dec(next("nlink", iter.next())?)?),
            b"nochange" => Keyword::NoChange,
            b"optional" => Keyword::Optional,
            b"resdevice" =>
                Keyword::ResidentDeviceRef(DeviceRef::from_bytes(next("resdevice", iter.next())?)?),
            b"rmd160" | b"rmd160digest" | b"ripemd160digest" =>
                Keyword::Rmd160(<[u8; 20]>::from_hex(
                        next("rmd160|rmd160digest|ripemd160digest", iter.next())?)?),
            b"sha1" | b"sha1digest" =>
                Keyword::Sha1(<[u8; 20]>::from_hex(next("sha1|sha1digest", iter.next())?)?),
            b"sha256" | b"sha256digest" =>
                Keyword::Sha256(<[u8; 32]>::from_hex(next("sha256|sha256digest", iter.next())?)?),
            b"sha384" | b"sha384digest" =>
                Keyword::Sha384(<[u8; 48]>::from_hex(next("sha384|sha384digest", iter.next())?)?),
            b"sha512" | b"sha512digest" =>
                Keyword::Sha512(<[u8; 64]>::from_hex(next("sha512|sha512digest", iter.next())?)?),
            b"size" => Keyword::Size(u64::from_dec(next("size", iter.next())?)?),
            b"time" => Keyword::Time(next("time", iter.next())?),
            b"type" => Keyword::Type(Type::from_bytes(next("type", iter.next())?)?),
            b"uid" => Keyword::Uid(u64::from_dec(next("uid", iter.next())?)?),
            b"uname" => Keyword::Uname(next("uname", iter.next())?),
            other => return Err(format!(r#""{}" is not a valid parameter key (in "{}")"#,
                                        String::from_utf8_lossy(other),
                                        String::from_utf8_lossy(input)
                                       ).into())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct DeviceRef<'a> {
    /// The device format
    format: Format,
    /// The device major identifier
    major: &'a [u8],
    /// The device minor identifier
    minor: &'a [u8],
    /// The device subunit identifier, if applicable.
    subunit: Option<&'a [u8]>,
}

impl<'a> DeviceRef<'a> {
    /// Take ownership of the underlying data by copying
    pub fn to_device(&self) -> Device {
        Device {
            format: self.format,
            major: self.major.to_owned(),
            minor: self.minor.to_owned(),
            subunit: self.subunit.map(|val| val.to_owned()),
        }
    }

    fn from_bytes(input: &'a [u8]) -> ParserResult<DeviceRef<'a>> {
        let mut iter = input.splitn(4, |ch| *ch == b',');
        let format = Format::from_bytes(iter.next().ok_or_else(|| {
            format!(r#"could not read format from device "{}""#, String::from_utf8_lossy(input))
        })?)?;
        let major = iter.next().ok_or_else(|| {
            format!(r#"could not read major field from device "{}""#,
                    String::from_utf8_lossy(input))
        })?;
        let minor = iter.next().ok_or_else(|| {
            format!(r#"could not read minor field from device "{}""#,
                    String::from_utf8_lossy(input))
        })?;
        // optional, so no '?'
        let subunit = iter.next();
        Ok(DeviceRef { format, major, minor, subunit })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Format {
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
}

impl Format {
    fn from_bytes(bytes: &[u8]) -> ParserResult<Format> {
        Ok(match bytes {
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
            ref other => return Err(format!(r#""{}" is not a valid format"#,
                                            String::from_utf8_lossy(other)).into()),
        })
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
    ] {
        assert_eq!(Format::from_bytes(&input[..]), Ok(res));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn from_bytes(input: &[u8]) -> ParserResult<Type> {
        Ok(match input {
            b"block" => Type::BlockDevice,
            b"char" => Type::CharacterDevice,
            b"dir" => Type::Directory,
            b"fifo" => Type::Fifo,
            b"file" => Type::File,
            b"link" => Type::SymbolicLink,
            b"socket" => Type::Socket,
            _ => return Err(format!(r#""{}" is not a valid file type"#,
                                    String::from_utf8_lossy(input)).into()),
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
        assert_eq!(Type::from_bytes(&input[..]), Ok(res));
    }
    assert!(Type::from_bytes(&b"other"[..]).is_err());
}

pub(crate) type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug, Eq, PartialEq, Fail)]
#[fail(display = "{}", _0)]
pub struct ParserError(String);

impl From<String> for ParserError {
    fn from(s: String) -> ParserError {
        ParserError(s)
    }
}


