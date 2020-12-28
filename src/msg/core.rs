/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, ClientID};

///A `core1.client-make` message.
///[\[vt6/core1, sect. X.Y\]](https://vt6.io/std/core1/#section-X-Y)
pub struct ClientMake<'a> {
    pub client_id: ClientID<'a>,
    pub stdin_screen_id: Option<&'a str>,
    pub stdout_screen_id: Option<&'a str>,
    pub stderr_screen_id: Option<&'a str>,
}

impl<'a> msg::DecodeMessage<'a> for ClientMake<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type().as_str() != "core1.client-make" {
            return None;
        }
        let (client_id, stdin_screen_id, stdout_screen_id, stderr_screen_id) =
            msg.arguments().exactly4()?;
        Some(ClientMake {
            client_id,
            stdin_screen_id,
            stdout_screen_id,
            stderr_screen_id,
        })
    }
}

impl<'a> msg::EncodeMessage for ClientMake<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "core1.client-make", 4);
        f.add_argument(&self.client_id);
        f.add_argument(&self.stdin_screen_id);
        f.add_argument(&self.stdout_screen_id);
        f.add_argument(&self.stderr_screen_id);
        f.finalize()
    }
}

///A `core1.client-new` message.
///[\[vt6/core1, sect. X.Y\]](https://vt6.io/std/core1/#section-X-Y)
pub struct ClientNew<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for ClientNew<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type().as_str() != "core1.client-new" {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(ClientNew { secret })
    }
}

impl<'a> msg::EncodeMessage for ClientNew<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "core1.client-new", 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}

///A `core1.client-end` message.
///[\[vt6/core1, sect. X.Y\]](https://vt6.io/std/core1/#section-X-Y)
pub struct ClientEnd<'a> {
    pub client_id: ClientID<'a>,
}

impl<'a> msg::DecodeMessage<'a> for ClientEnd<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type().as_str() != "core1.client-end" {
            return None;
        }
        let client_id = msg.arguments().exactly1()?;
        Some(ClientEnd { client_id })
    }
}

impl<'a> msg::EncodeMessage for ClientEnd<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "core1.client-end", 1);
        f.add_argument(&self.client_id);
        f.finalize()
    }
}
