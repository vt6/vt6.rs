/******************************************************************************
*
*  Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
*
******************************************************************************/

///A trait for types that can be encoded as an argument in a [VT6 message](msg/).
///
///The trait implementations for strings, byte strings and integers match the
///formats defined for basic property types in
///[vt6/core1.0, section 2.4](https://vt6.io/std/core/1.0/#section-2-4).
pub trait EncodeArgument {
    ///Returns the exact number of bytes that is required to encode this
    ///argument.
    fn get_size(&self) -> usize;
    ///Encodes this argument into the given buffer. The caller must ensure that
    ///the buffer is exactly `self.get_size()` bytes large.
    fn encode(&self, buf: &mut [u8]);
}

impl EncodeArgument for [u8] {
    fn get_size(&self) -> usize { self.len() }
    fn encode(&self, buf: &mut[u8]) { buf.copy_from_slice(self) }
}

impl EncodeArgument for str {
    fn get_size(&self) -> usize { self.len() }
    fn encode(&self, buf: &mut[u8]) { buf.copy_from_slice(self.as_bytes()) }
}

macro_rules! impl_EncodeArgument_for_integer {
    ($($t:ident),*: $t_conv:ident) => ($(
        //NOTE: Some of this is adapted from code in the Rust standard library
        //written primarily by Arthur Silva (@arthurprs on Github).
        //<https://github.com/rust-lang/rust/blob/329dde57fddee4d5fa0ae374cb5c8474459dfb0c/src/libcore/fmt/num.rs#L199>
        impl EncodeArgument for $t {
            fn get_size(&self) -> usize {
                #[allow(unused_comparisons)]
                let is_nonnegative = *self >= 0;
                let mut val = if is_nonnegative {
                    (*self as $t_conv)
                } else {
                    // convert the negative num to positive by summing 1 to it's 2 complement
                    (!(*self as $t_conv)).wrapping_add(1)
                };

                let mut result: usize = 1;
                while val > 9 {
                    result += 1;
                    val /= 10;
                }
                return result + if is_nonnegative { 0 } else { 1 /* for the minus sign */ };
            }

            fn encode(&self, buf: &mut[u8]) {
                #[allow(unused_comparisons)]
                let is_nonnegative = *self >= 0;
                let mut val = if is_nonnegative {
                    (*self as $t_conv)
                } else {
                    // convert the negative num to positive by summing 1 to it's 2 complement
                    (!(*self as $t_conv)).wrapping_add(1)
                };

                let mut idx = buf.len() - 1;
                while val > 9 {
                    buf[idx] = b'0' + ((val % 10) as u8);
                    val /= 10;
                    idx -= 1;
                }
                buf[idx] = b'0' + ((val % 10) as u8);

                if is_nonnegative {
                    assert!(idx == 0);
                } else {
                    assert!(idx == 1);
                    buf[0] = b'-';
                }
            }
        }
    )*);
}

//The smaller integer types (u8, u16) get upcast to u32 first because 32-bit
//arithmetic is typically faster.
impl_EncodeArgument_for_integer!(i8, u8, i16, u16, i32, u32: u32);
impl_EncodeArgument_for_integer!(i64, u64: u64);
impl_EncodeArgument_for_integer!(i128, u128: u128);
#[cfg(target_pointer_width = "16")]
impl_EncodeArgument_for_integer!(isize, usize: u16);
#[cfg(target_pointer_width = "32")]
impl_EncodeArgument_for_integer!(isize, usize: u32);
#[cfg(target_pointer_width = "64")]
impl_EncodeArgument_for_integer!(isize, usize: u64);

#[cfg(test)]
mod tests {

    use common::core::*;
    use std::str;
    use std::fmt::Display;

    fn check_encodes_like_display<T: EncodeArgument + Display + ?Sized>(val: &T) {
        let size = val.get_size();
        let mut buf = vec![0u8; size];
        val.encode(&mut buf);
        assert_eq!(str::from_utf8(&buf).unwrap(), format!("{}", val));
    }

    #[test]
    fn test_encode_strings() {
        check_encodes_like_display("abc");
        check_encodes_like_display("vt\u{1F4AF}=\u{1F4A9}");

        let val = b"abc = \xAA\xBB\xCC or something!";
        let mut buf = vec![0u8; val.get_size()];
        val.encode(&mut buf);
        assert_eq!(buf, val);
    }

    #[test]
    fn test_encode_unsigned() {
        check_encodes_like_display(&0u8);
        check_encodes_like_display(&42u8);
        check_encodes_like_display(&(u8::max_value() - 1));
        check_encodes_like_display(&(u8::max_value()));

        check_encodes_like_display(&0u16);
        check_encodes_like_display(&42u16);
        check_encodes_like_display(&(u16::max_value() - 1));
        check_encodes_like_display(&(u16::max_value()));

        check_encodes_like_display(&0u32);
        check_encodes_like_display(&42u32);
        check_encodes_like_display(&(u32::max_value() - 1));
        check_encodes_like_display(&(u32::max_value()));

        check_encodes_like_display(&0u64);
        check_encodes_like_display(&42u64);
        check_encodes_like_display(&(u64::max_value() - 1));
        check_encodes_like_display(&(u64::max_value()));

        check_encodes_like_display(&0u128);
        check_encodes_like_display(&42u128);
        check_encodes_like_display(&(u128::max_value() - 1));
        check_encodes_like_display(&(u128::max_value()));

        check_encodes_like_display(&0usize);
        check_encodes_like_display(&42usize);
        check_encodes_like_display(&(usize::max_value() - 1));
        check_encodes_like_display(&(usize::max_value()));
    }

    #[test]
    fn test_encode_signed() {
        check_encodes_like_display(&0i8);
        check_encodes_like_display(&-1i8);
        check_encodes_like_display(&42i8);
        check_encodes_like_display(&-42i8);
        check_encodes_like_display(&(i8::min_value()));
        check_encodes_like_display(&(i8::min_value() + 1));
        check_encodes_like_display(&(i8::max_value() - 1));
        check_encodes_like_display(&(i8::max_value()));

        check_encodes_like_display(&0i16);
        check_encodes_like_display(&-1i16);
        check_encodes_like_display(&42i16);
        check_encodes_like_display(&-42i16);
        check_encodes_like_display(&(i16::min_value()));
        check_encodes_like_display(&(i16::min_value() + 1));
        check_encodes_like_display(&(i16::max_value() - 1));
        check_encodes_like_display(&(i16::max_value()));

        check_encodes_like_display(&0i32);
        check_encodes_like_display(&-1i32);
        check_encodes_like_display(&42i32);
        check_encodes_like_display(&-42i32);
        check_encodes_like_display(&(i32::min_value()));
        check_encodes_like_display(&(i32::min_value() + 1));
        check_encodes_like_display(&(i32::max_value() - 1));
        check_encodes_like_display(&(i32::max_value()));

        check_encodes_like_display(&0i64);
        check_encodes_like_display(&-1i64);
        check_encodes_like_display(&42i64);
        check_encodes_like_display(&-42i64);
        check_encodes_like_display(&(i64::min_value()));
        check_encodes_like_display(&(i64::min_value() + 1));
        check_encodes_like_display(&(i64::max_value() - 1));
        check_encodes_like_display(&(i64::max_value()));

        check_encodes_like_display(&0i128);
        check_encodes_like_display(&-1i128);
        check_encodes_like_display(&42i128);
        check_encodes_like_display(&-42i128);
        check_encodes_like_display(&(i128::min_value()));
        check_encodes_like_display(&(i128::min_value() + 1));
        check_encodes_like_display(&(i128::max_value() - 1));
        check_encodes_like_display(&(i128::max_value()));

        check_encodes_like_display(&0isize);
        check_encodes_like_display(&-1isize);
        check_encodes_like_display(&42isize);
        check_encodes_like_display(&-42isize);
        check_encodes_like_display(&(isize::min_value()));
        check_encodes_like_display(&(isize::min_value() + 1));
        check_encodes_like_display(&(isize::max_value() - 1));
        check_encodes_like_display(&(isize::max_value()));
    }

    #[test]
    fn test_encode_module_version() {
        check_encodes_like_display(&ModuleVersion { major: 1, minor: 0 });
        check_encodes_like_display(&ModuleVersion { major: 23, minor: 42 });
        check_encodes_like_display(&ModuleVersion { major: 1, minor: u16::max_value() });
        check_encodes_like_display(&ModuleVersion { major: u16::max_value() - 1, minor: 2 });
    }

}
