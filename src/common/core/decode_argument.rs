/*******************************************************************************
*
* Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
* This program is free software: you can redistribute it and/or modify it under
* the terms of the GNU General Public License as published by the Free Software
* Foundation, either version 3 of the License, or (at your option) any later
* version.
*
* This program is distributed in the hope that it will be useful, but WITHOUT ANY
* WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
* A PARTICULAR PURPOSE. See the GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License along with
* this program. If not, see <http://www.gnu.org/licenses/>.
*
*******************************************************************************/

use std;

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
            _    => None,
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

                std::str::from_utf8(arg).ok()?.parse().ok()
            }
        }

    )*);
}

impl_DecodeArgument_for_integer!(
    i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);

#[cfg(test)]
mod tests {

    use common::core::*;

    #[test]
    fn test_decode_bool() {
        assert_eq!(bool::decode(b"t"),       Some(true));
        assert_eq!(bool::decode(b"f"),       Some(false));
        assert_eq!(bool::decode(b"unknown"), None);
        assert_eq!(bool::decode(b"1"),       None);
        assert_eq!(bool::decode(b"0"),       None);
        assert_eq!(bool::decode(b"true"),    None);
        assert_eq!(bool::decode(b"false"),   None);
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

    #[test]
    fn test_decode_moduleversion_fails() {
        assert_eq!(None, ModuleVersion::decode(b""));
        assert_eq!(None, ModuleVersion::decode(b"1"));
        assert_eq!(None, ModuleVersion::decode(b"1."));
        assert_eq!(None, ModuleVersion::decode(b".1"));
        assert_eq!(None, ModuleVersion::decode(b"1.2.3"));
        assert_eq!(None, ModuleVersion::decode(b".1.2"));
        assert_eq!(None, ModuleVersion::decode(b"1.2."));
        assert_eq!(None, ModuleVersion::decode(b"1..2"));
        assert_eq!(None, ModuleVersion::decode(b"1.abc"));
    }

}
