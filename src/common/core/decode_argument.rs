/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

///A trait for types that can be decoded from an argument in a [VT6 message](msg/).
///
///This is the inverse of [`trait EncodeArgument`](trait.EncodeArgument.html).
///
///The trait implementation for byte strings (`&[u8]`) is a no-op. It's only
///used by generic functions that decode arguments of arbitrary types.
///
///The trait implementations for booleans, integers and strings match the
///formats defined for basic property types in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///The generic trait implementation for `Option<T>` decodes empty inputs as
///`None` and anything else as `Some` (except for parse errors).
pub trait DecodeArgument<'a>: Sized {
    ///Parses a bytestring `s` (which is interpreted as an argument in a VT6
    ///message) into a value of this type. If parsing succeeds, `Some` is
    ///returned, otherwise `None` is returned.
    fn decode_argument(arg: &'a [u8]) -> Option<Self>;
}

impl<'a> DecodeArgument<'a> for bool {
    fn decode_argument(arg: &'a [u8]) -> Option<Self> {
        match arg {
            b"t" => Some(true),
            b"f" => Some(false),
            _ => None,
        }
    }
}

impl<'a> DecodeArgument<'a> for &'a [u8] {
    fn decode_argument(arg: &'a [u8]) -> Option<Self> {
        Some(arg)
    }
}

impl<'a> DecodeArgument<'a> for &'a str {
    fn decode_argument(arg: &'a [u8]) -> Option<Self> {
        core::str::from_utf8(arg).ok()
    }
}

impl<'a, T: DecodeArgument<'a>> DecodeArgument<'a> for Option<T> {
    fn decode_argument(arg: &'a [u8]) -> Option<Self> {
        if arg.is_empty() {
            Some(None)
        } else {
            Some(Some(T::decode_argument(arg)?))
        }
    }
}

macro_rules! impl_DecodeArgument_for_integer {
    ($($t:ty),*) => ($(

        impl<'a> DecodeArgument<'a> for $t {
            fn decode_argument(arg: &'a [u8]) -> Option<Self> {
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

macro_rules! impl_DecodeArgument_via_parse_from_string {
    ($($t:ty),*) => ($(

        impl<'a> DecodeArgument<'a> for $t {
            fn decode_argument(arg: &'a [u8]) -> Option<Self> {
                Self::parse(core::str::from_utf8(arg).ok()?)
            }
        }
    )*);
}

impl_DecodeArgument_for_integer!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);
impl_DecodeArgument_via_parse_from_string!(
    crate::common::core::ClientID<'a>,
    crate::common::core::Identifier<'a>,
    crate::common::core::MessageType<'a>,
    crate::common::core::ModuleIdentifier<'a>,
    crate::common::core::ModuleVersion<'a>,
    crate::common::core::ScopedIdentifier<'a>
);

#[cfg(test)]
mod tests {

    use crate::common::core::*;

    #[test]
    fn test_decode_bool() {
        assert_eq!(bool::decode_argument(b"t"), Some(true));
        assert_eq!(bool::decode_argument(b"f"), Some(false));
        assert_eq!(bool::decode_argument(b"unknown"), None);
        assert_eq!(bool::decode_argument(b"1"), None);
        assert_eq!(bool::decode_argument(b"0"), None);
        assert_eq!(bool::decode_argument(b"true"), None);
        assert_eq!(bool::decode_argument(b"false"), None);
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
            assert_eq!(None, i8::decode_argument(input));
            assert_eq!(None, u8::decode_argument(input));
            assert_eq!(None, i16::decode_argument(input));
            assert_eq!(None, u16::decode_argument(input));
            assert_eq!(None, i32::decode_argument(input));
            assert_eq!(None, u32::decode_argument(input));
            assert_eq!(None, i64::decode_argument(input));
            assert_eq!(None, u64::decode_argument(input));
            assert_eq!(None, i128::decode_argument(input));
            assert_eq!(None, u128::decode_argument(input));
            assert_eq!(None, isize::decode_argument(input));
            assert_eq!(None, usize::decode_argument(input));
        }
    }
}
