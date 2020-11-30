/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{ClientID, EncodeArgument};

///An encoding helper for client IDs, as defined by
///[vt6/foundation, section 2.6](https://vt6.io/std/foundation/#section-2-6).
///
///See documentation on [`ClientIDSuffix`](enum.ClientIDSuffix.html) for
///detailed explanation. Use
///[`ClientIDSuffix::below()`](enum.ClientIDSuffix.html#method.below) to
///construct instances of this type.
pub struct RelativeClientID<'a> {
    base: ClientID<'a>,
    suffix: ClientIDSuffix,
}

impl<'a> RelativeClientID<'a> {
    ///Returns the base client ID, i.e. the bytestring that was given to
    ///[`ClientIDSuffix::below()`](enum.ClientIDSuffix.html#method.below).
    pub fn base(&'_ self) -> ClientID<'a> {
        self.base
    }
    ///Returns the suffix that identifies this client ID relative to the base().
    pub fn suffix(&self) -> ClientIDSuffix {
        self.suffix
    }
}

impl<'a> EncodeArgument for RelativeClientID<'a> {
    fn get_size(&self) -> usize {
        self.base.get_size() + self.suffix.get_size()
    }

    fn encode(&self, buf: &mut [u8]) {
        let size = self.base.get_size();
        self.base.encode(&mut buf[0..size]);
        self.suffix.encode(&mut buf[size..]);
    }
}

///A specification of a client ID relative to this client's main client ID.
///(Client IDs are defined by
///[vt6/foundation, section 2.6](https://vt6.io/std/foundation/#section-2-6).)
///
///This type can be used to generate client IDs for process-local lifetimes and
///for child processes. To generate a client ID, write down the corresponding
///ClientIDSuffix instance, then use the `below()` method to prepend the current
///process's main client ID. This produces a RelativeClientID instance which
///implements EncodeArgument and encodes into the full client ID.
///
///TODO: Explain how to obtain the main client ID.
///
///In order to present an API that works for no_std clients, this type
///prescribes an encoding scheme for the client IDs of process-local lifetimes
///and child processes. The concrete encoding scheme is an implementation detail
///and not covered by any backwards-compatibility promises.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClientIDSuffix {
    ///No suffix at all. When appended to a client ID `base`, `Own.below(base)`
    ///encodes to `base`.
    Own,
    ///`Local(i)` is the client ID of the i-th process-local lifetime. It is up
    ///to the application to allocate indexes to specific lifetimes.
    Local(u32),
    ///`Job(i)` is the client ID of the i-th job spawned by this process. (A job
    ///is a set of child processes that share the same overall lifetime. In
    ///shells, each command line is a job, and the commands in it are children
    ///within that job.)
    ///
    ///It is okay to reuse job indexes as long as the `core1.lifetime-end`
    ///message has been sent for the job's lifetime before its index is reused.
    Job(u32),
    ///`Child(i, j)` is the client ID of the j-th child process in the i-th job
    ///spawned by this process.
    Child(u32, u32), //first = job index, second = child index in job
}
use self::ClientIDSuffix::*;

impl ClientIDSuffix {
    ///Prepends the given client ID to produce a full client ID that can be
    ///encoded into a message.
    pub fn below<'a>(&self, base: ClientID<'a>) -> RelativeClientID<'a> {
        RelativeClientID {
            base,
            suffix: *self,
        }
    }

    //This is an implementation of EncodeArgument, but we keep it private
    //because it's never useful to encode just a client ID suffix without the
    //base.
    //
    //encoding of Local(x)    = "0" + lookup[x]
    //encoding of Job(x)      = lookup[x]
    //encoding of Child(x, y) = lookup[x] + lookup[y]
    //  (see below for how lookup[x] is defined)
    //
    fn get_size(&self) -> usize {
        match *self {
            Own => 0,
            Local(i) => 1 + get_size_for_number(i),
            Job(i) => get_size_for_number(i),
            Child(i, j) => get_size_for_number(i) + get_size_for_number(j),
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        match *self {
            Own => {}
            Local(i) => {
                buf[0] = LOOKUP_TABLE[0];
                encode_number(i, &mut buf[1..]);
            }
            Job(i) => encode_number(i, buf),
            Child(i, j) => {
                let size = get_size_for_number(i);
                encode_number(i, &mut buf[0..size]);
                encode_number(j, &mut buf[size..]);
            }
        }
    }
}

//Definition for lookup[x] (mentioned above in the EncodeArgument implementation
//of ClientIDSuffix):
//
//  "1"
//  "2"
//  "3"
//  ... (iterate through digits, capital letters, then small letters) ...
//  "x"
//  "y"
//  "z0"
//  "z1"
//  "z2"
//  ...
//  "zx"
//  "zy"
//  "zz0"
//  "zz1"
//  "zz2"
//  ...
//
//  TODO: This is a very sparse encoding. There could be denser ones, but since
//  lookup[] is an implementation detail, we can just change it later.
fn get_size_for_number(num: u32) -> usize {
    //There are 10 + 2*26 - 1 = 61 codewords for each given length, except that the
    //codeword 0x00 is not used to avoid ambiguities in the ClientIDSuffix
    //encoding. Hence the `+ 1` below.
    (1 + (num + 1) / 61) as usize
}

const LOOKUP_TABLE: [u8; 62] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
];

fn encode_number(num: u32, buf: &mut [u8]) {
    //shift all numbers by 1 to account for the omitted codeword "0"
    let mut num = num + 1;
    for byte_mut in buf.iter_mut() {
        if num >= 61 {
            num -= 61;
            *byte_mut = LOOKUP_TABLE[61];
        } else {
            *byte_mut = LOOKUP_TABLE[num as usize];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientIDSuffix;
    use super::ClientIDSuffix::*;
    use crate::common::core::{ClientID, EncodeArgument};

    #[test]
    fn encode_decode_client_ids() {
        let base = ClientID::parse("foo").unwrap();
        let testcases: Vec<(ClientIDSuffix, &'static str)> = vec![
            (Own, "foo"),
            (Local(0), "foo01"),
            (Local(1), "foo02"),
            (Local(59), "foo0y"),
            (Local(60), "foo0z0"),
            (Local(61), "foo0z1"),
            (Local(120), "foo0zy"),
            (Local(121), "foo0zz0"),
            (Local(122), "foo0zz1"),
            (Job(0), "foo1"),
            (Job(1), "foo2"),
            (Job(59), "fooy"),
            (Job(60), "fooz0"),
            (Job(61), "fooz1"),
            (Job(120), "foozy"),
            (Job(121), "foozz0"),
            (Job(122), "foozz1"),
            (Child(0, 0), "foo11"),
            (Child(0, 1), "foo12"),
            (Child(0, 59), "foo1y"),
            (Child(0, 60), "foo1z0"),
            (Child(0, 61), "foo1z1"),
            (Child(0, 120), "foo1zy"),
            (Child(0, 121), "foo1zz0"),
            (Child(0, 122), "foo1zz1"),
            (Child(1, 0), "foo21"),
            (Child(1, 1), "foo22"),
            (Child(1, 59), "foo2y"),
            (Child(1, 60), "foo2z0"),
            (Child(1, 61), "foo2z1"),
            (Child(1, 120), "foo2zy"),
            (Child(1, 121), "foo2zz0"),
            (Child(1, 122), "foo2zz1"),
            (Child(61, 0), "fooz11"),
            (Child(61, 1), "fooz12"),
            (Child(61, 59), "fooz1y"),
            (Child(61, 60), "fooz1z0"),
            (Child(61, 61), "fooz1z1"),
            (Child(61, 120), "fooz1zy"),
            (Child(61, 121), "fooz1zz0"),
            (Child(61, 122), "fooz1zz1"),
        ];

        for (suffix, expected) in testcases {
            let encoded = suffix.below(base).encode_to_vector();
            assert_eq!(
                std::str::from_utf8(&encoded),
                Ok(expected),
                "suffix was: {:?}, encoded was: {:?}",
                suffix,
                encoded
            );
        }
    }
}
