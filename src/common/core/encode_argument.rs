/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

///A trait for types that can be encoded as an argument in a [VT6 message](msg/).
///
///This is the inverse of [`trait DecodeArgument`](trait.DecodeArgument.html).
///
///The trait implementations for strings, byte strings, booleans and integers
///match the formats defined for basic property types in
///[vt6/core1.0, section 2.4](https://vt6.io/std/core/1.0/#section-2-4).
///
///The generic trait implementation for `Option<>` encodes `Some(val)` just like
///`val` and `None` as an empty byte string. Because trait implementations in
///Rust must be unambiguous, there is no trait implementation for `Option<T>`,
///only for `Option<&T>`. If you have an `Option<T>`, use `Option::as_ref`.
///
///When the implementing type already contains a string representation of its encoding,
///[`trait EncodedArgument`](trait.EncodedArgument.html) can be implemented instead.
pub trait EncodeArgument {
    ///Returns the exact number of bytes that is required to encode this
    ///argument.
    fn get_size(&self) -> usize;
    ///Encodes this argument into the given buffer. The caller must ensure that
    ///the buffer is exactly `self.get_size()` bytes large.
    fn encode(&self, buf: &mut [u8]);

    ///A convenience function, mostly for usage in documentation examples, that
    ///allocates a Vec with the size indicated by get_size() and encodes the
    ///argument into it.
    #[cfg(any(test, feature = "use_std"))]
    fn encode_to_vector(&self) -> Vec<u8> {
        let mut v = vec![0u8; self.get_size()];
        self.encode(v.as_mut());
        v
    }
}
//NOTE(majewsky): I'm aware that this ^ is not the final design for this trait.
//It won't work as soon as we want to nest messages as arguments inside other
//messages (e.g. for multiplexing). To enable that usecase, we need an
//`impl<T> EncodeArgument for T where T: EncodeMessage`, which needs
//EncodeArgument, EncodeMessage and MessageFormatter to be more structurally
//similar.
//
//I'm kicking this particular can down the road in the hopes that
//<https://github.com/rust-lang/rust/issues/78485> will land before it becomes
//a problem. Once we can use std::io::ReadBuf, both traits could be redesigned as
//
//trait Encode... {
//    fn append_encoded_to(&self, buf: &mut std::io::ReadBuf) -> Result<(), BufferTooSmallError>;
//}

///A trait that simplifies the implementation of
///[`trait EncodeArgument`](trait.EncodeArgument.html) when the implementing type already contains
///a string representation of its encoding.
pub trait EncodedArgument {
    ///Returns the encoded form of this value for use as an argument in a VT6 message.
    fn encoded(&self) -> &[u8];
}

impl<T> EncodeArgument for T
where
    T: EncodedArgument + ?Sized,
{
    fn get_size(&self) -> usize {
        self.encoded().len()
    }
    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.encoded())
    }
}

impl EncodeArgument for [u8] {
    fn get_size(&self) -> usize {
        self.len()
    }
    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self)
    }
}

impl EncodeArgument for str {
    fn get_size(&self) -> usize {
        self.len()
    }
    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.as_bytes())
    }
}

#[cfg(feature = "use_std")]
impl EncodedArgument for std::path::Path {
    fn encoded(&self) -> &[u8] {
        use std::os::unix::ffi::OsStrExt;
        self.as_os_str().as_bytes()
    }
}

impl EncodeArgument for bool {
    fn get_size(&self) -> usize {
        1
    }
    fn encode(&self, buf: &mut [u8]) {
        assert_eq!(buf.len(), 1);
        buf[0] = if *self { b't' } else { b'f' };
    }
}

impl<'a, T> EncodeArgument for Option<&'a T>
where
    T: EncodeArgument + ?Sized,
{
    fn get_size(&self) -> usize {
        match *self {
            None => 0,
            Some(ref val) => val.get_size(),
        }
    }
    fn encode(&self, buf: &mut [u8]) {
        if let Some(ref val) = *self {
            val.encode(buf);
        }
    }
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

    use crate::common::core::*;
    use std::fmt::{Debug, Display};
    use std::str;

    fn check_encodes_like_display_and_decodes<
        T: EncodeArgument + for<'a> DecodeArgument<'a> + Display + Debug + Eq,
    >(
        val: &T,
    ) {
        check_encodes_like_display(val);

        let size = val.get_size();
        let mut buf = vec![0u8; size];
        val.encode(&mut buf);

        assert_eq!(Some(val), T::decode_argument(&buf).as_ref());
    }

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
    fn test_encode_bool() {
        let val = true;
        let mut buf = vec![0u8; val.get_size()];
        val.encode(&mut buf);
        assert_eq!(buf, b"t");

        let val = false;
        let mut buf = vec![0u8; val.get_size()];
        val.encode(&mut buf);
        assert_eq!(buf, b"f");
    }

    #[test]
    fn test_encode_unsigned() {
        check_encodes_like_display_and_decodes(&0u8);
        check_encodes_like_display_and_decodes(&42u8);
        check_encodes_like_display_and_decodes(&(u8::max_value() - 1));
        check_encodes_like_display_and_decodes(&(u8::max_value()));

        check_encodes_like_display_and_decodes(&0u16);
        check_encodes_like_display_and_decodes(&42u16);
        check_encodes_like_display_and_decodes(&(u16::max_value() - 1));
        check_encodes_like_display_and_decodes(&(u16::max_value()));

        check_encodes_like_display_and_decodes(&0u32);
        check_encodes_like_display_and_decodes(&42u32);
        check_encodes_like_display_and_decodes(&(u32::max_value() - 1));
        check_encodes_like_display_and_decodes(&(u32::max_value()));

        check_encodes_like_display_and_decodes(&0u64);
        check_encodes_like_display_and_decodes(&42u64);
        check_encodes_like_display_and_decodes(&(u64::max_value() - 1));
        check_encodes_like_display_and_decodes(&(u64::max_value()));

        check_encodes_like_display_and_decodes(&0u128);
        check_encodes_like_display_and_decodes(&42u128);
        check_encodes_like_display_and_decodes(&(u128::max_value() - 1));
        check_encodes_like_display_and_decodes(&(u128::max_value()));

        check_encodes_like_display_and_decodes(&0usize);
        check_encodes_like_display_and_decodes(&42usize);
        check_encodes_like_display_and_decodes(&(usize::max_value() - 1));
        check_encodes_like_display_and_decodes(&(usize::max_value()));
    }

    #[test]
    fn test_encode_signed() {
        check_encodes_like_display_and_decodes(&0i8);
        check_encodes_like_display_and_decodes(&-1i8);
        check_encodes_like_display_and_decodes(&42i8);
        check_encodes_like_display_and_decodes(&-42i8);
        check_encodes_like_display_and_decodes(&(i8::min_value()));
        check_encodes_like_display_and_decodes(&(i8::min_value() + 1));
        check_encodes_like_display_and_decodes(&(i8::max_value() - 1));
        check_encodes_like_display_and_decodes(&(i8::max_value()));

        check_encodes_like_display_and_decodes(&0i16);
        check_encodes_like_display_and_decodes(&-1i16);
        check_encodes_like_display_and_decodes(&42i16);
        check_encodes_like_display_and_decodes(&-42i16);
        check_encodes_like_display_and_decodes(&(i16::min_value()));
        check_encodes_like_display_and_decodes(&(i16::min_value() + 1));
        check_encodes_like_display_and_decodes(&(i16::max_value() - 1));
        check_encodes_like_display_and_decodes(&(i16::max_value()));

        check_encodes_like_display_and_decodes(&0i32);
        check_encodes_like_display_and_decodes(&-1i32);
        check_encodes_like_display_and_decodes(&42i32);
        check_encodes_like_display_and_decodes(&-42i32);
        check_encodes_like_display_and_decodes(&(i32::min_value()));
        check_encodes_like_display_and_decodes(&(i32::min_value() + 1));
        check_encodes_like_display_and_decodes(&(i32::max_value() - 1));
        check_encodes_like_display_and_decodes(&(i32::max_value()));

        check_encodes_like_display_and_decodes(&0i64);
        check_encodes_like_display_and_decodes(&-1i64);
        check_encodes_like_display_and_decodes(&42i64);
        check_encodes_like_display_and_decodes(&-42i64);
        check_encodes_like_display_and_decodes(&(i64::min_value()));
        check_encodes_like_display_and_decodes(&(i64::min_value() + 1));
        check_encodes_like_display_and_decodes(&(i64::max_value() - 1));
        check_encodes_like_display_and_decodes(&(i64::max_value()));

        check_encodes_like_display_and_decodes(&0i128);
        check_encodes_like_display_and_decodes(&-1i128);
        check_encodes_like_display_and_decodes(&42i128);
        check_encodes_like_display_and_decodes(&-42i128);
        check_encodes_like_display_and_decodes(&(i128::min_value()));
        check_encodes_like_display_and_decodes(&(i128::min_value() + 1));
        check_encodes_like_display_and_decodes(&(i128::max_value() - 1));
        check_encodes_like_display_and_decodes(&(i128::max_value()));

        check_encodes_like_display_and_decodes(&0isize);
        check_encodes_like_display_and_decodes(&-1isize);
        check_encodes_like_display_and_decodes(&42isize);
        check_encodes_like_display_and_decodes(&-42isize);
        check_encodes_like_display_and_decodes(&(isize::min_value()));
        check_encodes_like_display_and_decodes(&(isize::min_value() + 1));
        check_encodes_like_display_and_decodes(&(isize::max_value() - 1));
        check_encodes_like_display_and_decodes(&(isize::max_value()));
    }
}
