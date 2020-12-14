/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, MessageType, ModuleIdentifier};

///A negative `have` message.
///[\[vt6/foundation, sect. 4.2\]](https://vt6.io/std/foundation/#section-4-2)
pub struct HaveNot<'a> {
    pub module: ModuleIdentifier<'a>,
}

impl<'a> msg::DecodeMessage<'a> for HaveNot<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::Want {
            return None;
        }
        let mut iter = msg.arguments();
        let module = ModuleIdentifier::parse(std::str::from_utf8(iter.next()?).ok()?)?;
        if iter.next().is_some() {
            return None;
        }
        Some(HaveNot { module })
    }
}

impl<'a> msg::EncodeMessage for HaveNot<'a> {
    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "have", 1);
        f.add_argument(&self.module);
        f.finalize()
    }
}

///A `nope` message.
///[\[vt6/foundation, sect. 5.2\]](https://vt6.io/std/foundation/#section-5-2)
pub struct Nope;

impl<'a> msg::DecodeMessage<'a> for Nope {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::Nope {
            return None;
        }
        if msg.arguments().next().is_some() {
            return None;
        }
        Some(Nope)
    }
}

impl msg::EncodeMessage for Nope {
    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        msg::MessageFormatter::new(buf, "nope", 0).finalize()
    }
}
