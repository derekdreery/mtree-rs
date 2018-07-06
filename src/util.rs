//! Utility misc stuff
use parser::{ParserResult, ParserError};
use std::time::Duration;

pub trait FromHex: Sized {
    fn from_hex(input: &[u8]) -> ParserResult<Self>;
}

pub trait FromDec: Sized {
    fn from_dec(input: &[u8]) -> ParserResult<Self>;
}

macro_rules! impl_FromDec_uint {
    ($from:ty) => {
        impl FromDec for $from {
            fn from_dec(input: &[u8]) -> ParserResult<Self> {
                let mut acc: Self = 0;
                for (idx, i) in input.iter().enumerate() {
                    let val = from_dec_ch(*i).ok_or_else(|| {
                        format!(r#"could not parse "{}" as a number, problem at char {}"#,
                                String::from_utf8_lossy(input),
                                idx)})?;
                    acc = acc.checked_mul(10).unwrap().checked_add(val as $from).unwrap();
                }
                Ok(acc)
            }
        }
    }
}

impl_FromDec_uint!(u8);
impl_FromDec_uint!(u16);
impl_FromDec_uint!(u32);
impl_FromDec_uint!(u64);

macro_rules! impl_FromHex_arr {
    ($size:expr) => {
        impl FromHex for [u8; $size] {
            fn from_hex(input: &[u8]) -> ParserResult<Self> {
                if input.len() != 2 * $size {
                    return Err(format!(r#"input length ({}) must be twice the vec size ({}), but \
                        it is not (in "{}")"#, input.len(), $size,
                        String::from_utf8_lossy(input)).into());
                }
                let mut acc = [0; $size];
                for (idx, chunk) in input.chunks(2).enumerate() {
                    let high = from_hex_ch(chunk[0]).ok_or_else(|| {
                        format!(r#"char at position {} in "{}" is not hex"#,
                                2 * idx,
                                String::from_utf8_lossy(input))
                    })?;
                    let low = from_hex_ch(chunk[1]).ok_or_else(|| {
                        format!(r#"char at position {} in "{}" is not hex"#,
                                2 * idx + 1,
                                String::from_utf8_lossy(input))
                    })?;
                    acc[idx] = high * 16 + low;
                }
                Ok(acc)
            }
        }
    }
}

impl_FromHex_arr!(16);
impl_FromHex_arr!(20);
impl_FromHex_arr!(32);

macro_rules! impl_FromHex_newtype {
    ($type:ty, $size:expr) => {
        impl FromHex for $type {
            fn from_hex(input: &[u8]) -> ParserResult<Self> {
                if input.len() != 2 * $size {
                    return Err(format!(r#"input length ({}) must be twice the vec size ({}), but \
                        it is not (in "{}")"#, input.len(), $size,
                        String::from_utf8_lossy(input)).into());
                }
                let mut acc = [0; $size];
                for (idx, chunk) in input.chunks(2).enumerate() {
                    let high = from_hex_ch(chunk[0]).ok_or_else(|| {
                        format!(r#"char at position {} in "{}" is not hex"#,
                                2 * idx,
                                String::from_utf8_lossy(input))
                    })?;
                    let low = from_hex_ch(chunk[1]).ok_or_else(|| {
                        format!(r#"char at position {} in "{}" is not hex"#,
                                2 * idx + 1,
                                String::from_utf8_lossy(input))
                    })?;
                    acc[idx] = high * 16 + low;
                }
                Ok(acc.into())
            }
        }
    }
}

impl_FromHex_newtype!(Array48<u8>, 48);
impl_FromHex_newtype!(Array64<u8>, 64);

macro_rules! impl_FromHex_uint {
    ($from:ty) => {
        impl FromHex for $from {
            fn from_hex(input: &[u8]) -> ParserResult<Self> {
                let mut acc: Self = 0;
                for (idx, i) in input.iter().enumerate() {
                    let val = from_hex_ch(*i).ok_or_else(|| {
                        format!(r#"could not parse "{}" as a number, problem at char {}"#,
                                String::from_utf8_lossy(input),
                                idx)})?;
                    acc = acc.checked_mul(16).unwrap().checked_add(val as $from).unwrap();
                }
                Ok(acc)
            }
        }
    }
}

impl_FromHex_uint!(u128);

fn from_hex_ch(i: u8) -> Option<u8> {
   Some(match i {
       b'0' => 0,
       b'1' => 1,
       b'2' => 2,
       b'3' => 3,
       b'4' => 4,
       b'5' => 5,
       b'6' => 6,
       b'7' => 7,
       b'8' => 8,
       b'9' => 9,
       b'a' | b'A'  => 10,
       b'b' | b'B'  => 11,
       b'c' | b'C'  => 12,
       b'd' | b'D'  => 13,
       b'e' | b'E'  => 14,
       b'f' | b'F'  => 15,
       _ => return None,
   })
}

fn from_dec_ch(i: u8) -> Option<u8> {
   Some(match i {
       b'0' => 0,
       b'1' => 1,
       b'2' => 2,
       b'3' => 3,
       b'4' => 4,
       b'5' => 5,
       b'6' => 6,
       b'7' => 7,
       b'8' => 8,
       b'9' => 9,
       _ => return None,
   })
}

pub fn from_oct_ch(i: u8) -> Option<u8> {
   Some(match i {
       b'0' => 0,
       b'1' => 1,
       b'2' => 2,
       b'3' => 3,
       b'4' => 4,
       b'5' => 5,
       b'6' => 6,
       b'7' => 7,
       _ => return None,
   })
}

pub fn parse_time(input: &[u8]) -> ParserResult<Duration> {
    let error = || -> ParserError {
        format!(r#"couldn't parse time from "{}""#, String::from_utf8_lossy(input)).into()
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

/*
impl<T: fmt::LowerHex> fmt::Display for Array48<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for el in &self[..] {
            write!(f, "{:x}", el)?;
        }
        Ok(())
    }
}

impl<T: fmt::LowerHex> fmt::Display for Array64<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for el in &self[..] {
            write!(f, "{:x}", el)?;
        }
        Ok(())
    }
}
*/
