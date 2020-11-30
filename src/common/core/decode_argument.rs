/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

///A trait for types that can be decoded from an argument in a [VT6 message](msg/).
///
///This is the inverse of [`trait EncodeArgument`](trait.EncodeArgument.html).
///
///A notable difference is that this trait is *not* implemented for strings and
///bytestrings because neither `[u8]` nor `str` are `Sized`. To decode an
///argument into a `&str`, use `std::str::from_utf8()`. If `&[u8]` is the
///desired value type, no decoding is required.
///
///The trait implementations for booleans and integers match the formats defined
///for basic property types in
///[vt6/core1.0, section 2.4](https://vt6.io/std/core/1.0/#section-2-4).
pub trait DecodeArgument: Sized {
    ///Parses a bytestring `s` (which is interpreted as an argument in a VT6
    ///message) into a value of this type. If parsing succeeds, `Some` is
    ///returned, otherwise `None` is returned.
    fn decode(arg: &[u8]) -> Option<Self>;
}

impl DecodeArgument for bool {
    fn decode(arg: &[u8]) -> Option<Self> {
        match arg {
            b"t" => Some(true),
            b"f" => Some(false),
            _ => None,
        }
    }
}

macro_rules! impl_DecodeArgument_for_integer {
    ($($t:ident),*) => ($(

        impl DecodeArgument for $t {
            fn decode(arg: &[u8]) -> Option<Self> {
                //forbid leading zeroes
                if arg.len() == 0 {
                    return None;
                }
                if arg != b"0" && arg[0] == b'0' {
                    return None;
                }

                core::str::from_utf8(arg).ok()?.parse().ok()
            }
        }

    )*);
}

impl_DecodeArgument_for_integer!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);

#[cfg(test)]
mod tests {

    use crate::common::core::*;

    #[test]
    fn test_decode_bool() {
        assert_eq!(bool::decode(b"t"), Some(true));
        assert_eq!(bool::decode(b"f"), Some(false));
        assert_eq!(bool::decode(b"unknown"), None);
        assert_eq!(bool::decode(b"1"), None);
        assert_eq!(bool::decode(b"0"), None);
        assert_eq!(bool::decode(b"true"), None);
        assert_eq!(bool::decode(b"false"), None);
    }

    //NOTE: The tests below only test error cases (where `decode(...)` returns
    //None), since the positive cases are covered in encode_argument.rs, where
    //it is checked if `decode(encode(x)) == x`.

    #[test]
    fn test_decode_u8_fails() {
        let invalid_inputs: Vec<&'static [u8]> = vec![
            b"",
            b"unknown",
            b"\xC0\xB1", //UTF-8 overlong encoding of "1"
            b" 42",      //whitespace in front
            b"042",      //zeroes in front
            b"0042",     //more zeroes in front
        ];

        for input in invalid_inputs {
            assert_eq!(None, i8::decode(input));
            assert_eq!(None, u8::decode(input));
            assert_eq!(None, i16::decode(input));
            assert_eq!(None, u16::decode(input));
            assert_eq!(None, i32::decode(input));
            assert_eq!(None, u32::decode(input));
            assert_eq!(None, i64::decode(input));
            assert_eq!(None, u64::decode(input));
            assert_eq!(None, i128::decode(input));
            assert_eq!(None, u128::decode(input));
            assert_eq!(None, isize::decode(input));
            assert_eq!(None, usize::decode(input));
        }
    }
}
