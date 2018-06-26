//! Stuff for parsing mtree files.

/// An mtree file is a sequence of lines, each a semantic unit.
#[derive(Debug, PartialEq, Eq)]
enum MTreeLine<'a> {
    /// Blank lines are ignored.
    Blank,
    /// Lines starting with a '#' are ignored.
    Comment(&'a [u8]),
    /// Special commands (starting with '/') alter the behavior of later entries.
    Special(Special),
    /// If the first word does not contain a '/', it is a file in the current
    /// directory.
    Relative(&'a [u8]),
    /// Change the current directory to the parent of the current directory.
    DotDot,
    /// If the first word does contain a '/', it is a file relative to the starting
    /// (not current) directory.
    Full(&'a [u8]),
}

/// A command that alters the behavior of later commands.
#[derive(Debug, PartialEq, Eq)]
enum Special {
    /// Set a default for future lines.
    Set,
    /// Unset a default for future lines.
    Unset,
}

/// Each entry may have one or more key word
#[derive(Debug, PartialEq, Eq)]
enum Keyword<'a> {
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
    Inode(&'a [u8]),
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

impl<'a> Keyword<'a> {
    /// Parse a keyword with optional value.
    fn from_bytes(input: &'a [u8]) -> Option<Keyword<'a>> {
        let mut iter = input.splitn(2, |ch| *ch == b'=');
        let key = iter.next()?;
        Ok(match key {
            b"cksum" => Keyword::Checksum(iter.next()?),
            b"device" => Keyword::Device(Device::from_bytes(iter.next()?)?),
            b"contents" => Keyword::Contents(iter.next()?),
            b"flags" => Keyword::Flags(iter.next()?),
            b"gid" => Keyword::Gid(u64::from_dec(iter.next()?)?),
            b"gname" => Keyword::Gname(iter.next()?),
            b"ignore" => Keyword::Ignore,
            b"inode" => Keyword::Inode(u64::from_dic(iter.next()?)?),
            b"link" => Keyword::Link(iter.next()?),
            b"md5" | b"md5digest" => Keyword::Md5([u8; 16]),
            b"mode" => Keyword::Mode(&'a [u8]),
            b"nlink" => Keyword::NLink(u64),
            b"nochange" => Keyword::NoChange,
            b"optional" => Keyword::Optional,
            b"resdevice" => Keyword::ResidentDevice(Device<'a>),
            b"rmd160" | b"rmd160digest" | b"ripemd160digest" => {
                Keyword::Rmd160([u8; 20])
            },
            b"sha1" | b"sha1digest" => Keyword::Sha1([u8; 20]),
            b"sha256" | b"sha256digest" => Keyword::Sha256([u8; 32]),
            b"sha384" | b"sha384digest" => Keyword::Sha384([u8; 48]),
            b"sha512" | b"sha512digest" => Keyword::Sha512([u8; 64]),
            b"size" => Keyword::Size(u64),
            b"time" => Keyword::Time(&'a [u8]),
            b"type" => Keyword::Type(Type),
            b"uid" => Keyword::Uid(u64::from_dec_bytes(input)),
            b"uname" => Keyword::Uname(u64::from_dec_bytes(input)),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Device<'a> {
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

#[derive(Debug, PartialEq, Eq)]
enum Format<'a> {
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
enum Type {
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
