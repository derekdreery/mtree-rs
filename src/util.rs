//! Utility misc stuff
use crate::parser::{ParserError, ParserResult};
use std::time::Duration;

/// Helper to parse a number from a slice of u8 in hexadecimal.
pub trait FromHex: Sized {
    /// Parse a number from a slice of u8 in hexadecimal.
    fn from_hex(input: &[u8]) -> ParserResult<Self>;
}

/// Helper to parse a number from a slice of u8 in decimal.
pub trait FromDec: Sized {
    /// Parse a number from a slice of u8 in decimal.
    fn from_dec(input: &[u8]) -> ParserResult<Self>;
}

macro_rules! impl_FromDec_uint {
    ($from:ty) => {
        impl FromDec for $from {
            fn from_dec(input: &[u8]) -> ParserResult<Self> {
                let mut acc: Self = 0;
                for (idx, i) in input.iter().enumerate() {
                    let val = from_dec_ch(*i).ok_or_else(|| {
                        format!(
                            r#"could not parse "{}" as a number, problem at char {}"#,
                            String::from_utf8_lossy(input),
                            idx
                        )
                    })?;
                    acc = acc
                        .checked_mul(10)
                        .ok_or_else(|| {
                            ParserError::from("could not parse integer - shift overflow".to_owned())
                        })?
                        .checked_add(val as $from)
                        .ok_or_else(|| {
                            ParserError::from(
                                "could not parse integer - addition overflow".to_owned(),
                            )
                        })?;
                }
                Ok(acc)
            }
        }
    };
}

impl_FromDec_uint!(u8);
impl_FromDec_uint!(u16);
impl_FromDec_uint!(u32);
impl_FromDec_uint!(u64);

macro_rules! impl_FromHex_arr {
    ($size:expr) => {
        impl FromHex for [u8; $size] {
            #[inline]
            fn from_hex(input: &[u8]) -> ParserResult<Self> {
                if input.len() != 2 * $size {
                    return Err(format!(
                        r#"input length ({}) must be twice the vec size ({}), but \
                                                                        it is not (in "{}")"#,
                        input.len(),
                        $size,
                        String::from_utf8_lossy(input)
                    )
                    .into());
                }
                let mut acc = [0; $size];
                for (idx, chunk) in input.chunks(2).enumerate() {
                    let high = from_hex_ch(chunk[0]).ok_or_else(|| {
                        format!(
                            r#"char at position {} in "{}" is not hex"#,
                            2 * idx,
                            String::from_utf8_lossy(input)
                        )
                    })?;
                    let low = from_hex_ch(chunk[1]).ok_or_else(|| {
                        format!(
                            r#"char at position {} in "{}" is not hex"#,
                            2 * idx + 1,
                            String::from_utf8_lossy(input)
                        )
                    })?;
                    acc[idx] = high * 16 + low;
                }
                Ok(acc)
            }
        }
    };
}

impl_FromHex_arr!(16);
impl_FromHex_arr!(20);
impl_FromHex_arr!(32);

macro_rules! impl_FromHex_newtype {
    ($type:ty, $size:expr) => {
        impl FromHex for $type {
            #[inline]
            fn from_hex(input: &[u8]) -> ParserResult<Self> {
                if input.len() != 2 * $size {
                    return Err(format!(
                        r#"input length ({}) must be twice the vec size ({}), but \
                                                                        it is not (in "{}")"#,
                        input.len(),
                        $size,
                        String::from_utf8_lossy(input)
                    )
                    .into());
                }
                let mut acc = [0; $size];
                for (idx, chunk) in input.chunks(2).enumerate() {
                    let high = from_hex_ch(chunk[0]).ok_or_else(|| {
                        format!(
                            r#"char at position {} in "{}" is not hex"#,
                            2 * idx,
                            String::from_utf8_lossy(input)
                        )
                    })?;
                    let low = from_hex_ch(chunk[1]).ok_or_else(|| {
                        format!(
                            r#"char at position {} in "{}" is not hex"#,
                            2 * idx + 1,
                            String::from_utf8_lossy(input)
                        )
                    })?;
                    acc[idx] = high * 16 + low;
                }
                Ok(acc.into())
            }
        }
    };
}

impl_FromHex_newtype!(Array48<u8>, 48);
impl_FromHex_newtype!(Array64<u8>, 64);

impl FromHex for u128 {
    /// Convert hex to u128
    ///
    /// # Panics
    ///
    /// The input length must be exactly 32.
    #[inline]
    fn from_hex(input: &[u8]) -> ParserResult<Self> {
        if input.len() != 32 {
            return Err(format!(
                r#"could not parse "{}" as a number, must be 32 chars"#,
                String::from_utf8_lossy(input)
            )
            .into());
        }
        let mut acc: Self = 0;
        for (idx, i) in input.iter().enumerate() {
            let val = from_hex_ch(*i).ok_or_else(|| {
                format!(
                    r#"could not parse "{}" as a number, problem at char {}"#,
                    String::from_utf8_lossy(input),
                    idx
                )
            })?;
            acc = acc * 16 + val as u128;
        }
        Ok(acc)
    }
}

/// If possible, quickly convert a character of a hexadecimal number into a u8.
#[inline]
fn from_hex_ch(i: u8) -> Option<u8> {
    match i {
        b'0'..=b'9' => Some(i - b'0'),
        b'a'..=b'f' => Some(i - b'a' + 10),
        b'A'..=b'F' => Some(i - b'A' + 10),
        _ => None,
    }
}

/// If possible, quickly convert a character of a decimal number into a u8.
#[inline]
fn from_dec_ch(i: u8) -> Option<u8> {
    match i {
        b'0'..=b'9' => Some(i - b'0'),
        _ => None,
    }
}

/// If possihble, quickly convert a character of a hexadecimal number into a u8.
#[inline]
pub fn from_oct_ch(i: u8) -> Option<u8> {
    match i {
        b'0'..=b'7' => Some(i - b'0'),
        _ => None,
    }
}

/// Convert a time of format `<seconds>.<nanos>` into a rust `Duration`.
pub fn parse_time(input: &[u8]) -> ParserResult<Duration> {
    let error = || -> ParserError {
        format!(
            r#"couldn't parse time from "{}""#,
            String::from_utf8_lossy(input)
        )
        .into()
    };
    let mut time_iter = input.splitn(2, |ch| *ch == b'.');
    let sec = time_iter.next().ok_or_else(error)?;
    let sec = u64::from_dec(sec)?;
    let nano = time_iter.next().ok_or_else(error)?;
    let nano = u32::from_dec(nano)?;
    Ok(Duration::new(sec, nano))
}

newtype_array!(pub struct Array48(48));
newtype_array!(pub struct Array64(64));
