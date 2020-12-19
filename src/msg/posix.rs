/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;

///A `posix1.stdin-hello` message.
///[\[vt6/posix1, sect. X.Y\]](https://vt6.io/std/posix1/#section-X-Y)
pub struct StdinHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for StdinHello<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type().as_str() != "posix1.stdin-hello" {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(StdinHello { secret })
    }
}

impl<'a> msg::EncodeMessage for StdinHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "posix1.stdin-hello", 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}

///A `posix1.stdout-hello` message.
///[\[vt6/posix1, sect. X.Y\]](https://vt6.io/std/posix1/#section-X-Y)
pub struct StdoutHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for StdoutHello<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type().as_str() != "posix1.stdout-hello" {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(StdoutHello { secret })
    }
}

impl<'a> msg::EncodeMessage for StdoutHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "posix1.stdout-hello", 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}
