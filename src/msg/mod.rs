/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, DecodeArgument, MessageType, ModuleIdentifier, ModuleVersion};

///Message types for the [vt6/core](https://vt6.io/std/core/) module.
pub mod core;
///Message types for the [vt6/posix](https://vt6.io/std/posix/) module.
pub mod posix;

///A `want` message.
///[\[vt6/foundation, sect. 4.1\]](https://vt6.io/std/foundation/#section-4-1)
pub struct Want<'a>(pub ModuleIdentifier<'a>);

impl<'a> msg::DecodeMessage<'a> for Want<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::Want {
            return None;
        }
        let ident = msg.arguments().exactly1()?;
        Some(Want(ident))
    }
}

impl<'a> msg::EncodeMessage for Want<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "want", 1);
        f.add_argument(&self.0);
        f.finalize()
    }
}

///A `have` message.
///[\[vt6/foundation, sect. 4.2\]](https://vt6.io/std/foundation/#section-4-2)
pub enum Have<'a> {
    ThisModule(ModuleVersion<'a>),
    NotThisModule(ModuleIdentifier<'a>),
}

impl<'a> msg::DecodeMessage<'a> for Have<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::Have {
            return None;
        }
        let arg: &'a [u8] = msg.arguments().exactly1()?;
        if let Some(version) = ModuleVersion::decode_argument(arg) {
            Some(Have::ThisModule(version))
        } else if let Some(module) = ModuleIdentifier::decode_argument(arg) {
            Some(Have::NotThisModule(module))
        } else {
            None
        }
    }
}

impl<'a> msg::EncodeMessage for Have<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "have", 1);
        match *self {
            Have::ThisModule(ref v) => f.add_argument(v),
            Have::NotThisModule(ref m) => f.add_argument(m),
        };
        f.finalize()
    }
}

///A `nope` message.
///[\[vt6/foundation, sect. 5.2\]](https://vt6.io/std/foundation/#section-5-2)
pub struct Nope<'a>(pub MessageType<'a>);

impl<'a> msg::DecodeMessage<'a> for Nope<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::Nope {
            return None;
        }
        let mt = msg.arguments().exactly1()?;
        Some(Nope(mt))
    }
}

impl<'a> msg::EncodeMessage for Nope<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "nope", 1);
        f.add_argument(&self.0);
        f.finalize()
    }
}
