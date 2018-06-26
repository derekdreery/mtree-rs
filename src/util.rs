//! Utility misc stuff

pub trait FromHex: Sized {
    fn from_hex(input: &[u8]) -> Option<Self>;
}

pub trait FromDec: Sized {
    fn from_dec(input: &[u8]) -> Option<Self>;
}

macro_rules! impl_FromDec_uint {
    ($from:ty) => {
        impl FromDec for $from {
            fn from_dec(input: &[u8]) -> Option<Self> {
                let mut acc = 0;
                for i in input {
                    acc = acc * 10 + from_dec_ch(*i)? as $from;
                }
                Some(acc)
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
            fn from_hex(input: &[u8]) -> Option<Self> {
                if input.len() != 2 * $size {
                    return None;
                }
                let mut acc = [0; $size];
                for (idx, chunk) in input.chunks(2).enumerate() {
                    let num = from_hex_ch(chunk[0])? * 16 + from_hex_ch(chunk[1])?;
                    acc[idx] = num;
                }
                Some(acc)
            }
        }
    }
}

impl_FromHex_arr!(16);
impl_FromHex_arr!(20);
impl_FromHex_arr!(32);
impl_FromHex_arr!(48);
impl_FromHex_arr!(64);

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

