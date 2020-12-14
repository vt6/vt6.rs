/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;

///A trait for types that contain a parsed form of a VT6 message.
///
///This is the inverse of [`trait EncodeMessage`](trait.EncodeMessage.html).
///
///For most messages defined in the main VT6 modules, there is a message type implementing this
///trait in [vt6::msg](../../../msg/index.html).
pub trait DecodeMessage<'a>: Sized {
    fn decode_message(msg: &'a msg::Message) -> Option<Self>;
}

///A trait for types that serialize into a VT6 message.
///
///This is the inverse of [`trait EncodeMessage`](trait.EncodeMessage.html).
///
///For most messages defined in the main VT6 modules, there is a message type implementing this
///trait in [vt6::msg](../../../msg/index.html).
pub trait EncodeMessage {
    ///As the signature suggests, implementations of this method commonly use a
    ///[MessageFormatter](struct.MessageFormatter.html) to do the encoding work.
    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError>;
}
