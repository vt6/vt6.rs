/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{
    msg, ClientID, DecodeArgument, MessageType, ModuleIdentifier, ModuleVersion,
};

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
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        msg::MessageFormatter::new(buf, "nope", 0).finalize()
    }
}

///A `client-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
pub struct ClientHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for ClientHello<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::ClientHello {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(ClientHello { secret })
    }
}

impl<'a> msg::EncodeMessage for ClientHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "client-hello", 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}

///A `parent-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
pub struct ParentHello<'a> {
    pub client_secret: &'a str,
    pub server_uri: &'a [u8],
}

impl<'a> msg::DecodeMessage<'a> for ParentHello<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::ParentHello {
            return None;
        }
        let (client_secret, server_uri) = msg.arguments().exactly2()?;
        Some(ParentHello {
            client_secret,
            server_uri,
        })
    }
}

impl<'a> msg::EncodeMessage for ParentHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "parent-hello", 2);
        f.add_argument(self.client_secret);
        f.add_argument(self.server_uri);
        f.finalize()
    }
}

///A `server-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
pub struct ServerHello<'a> {
    pub client_id: ClientID<'a>,
    pub stdin_screen_id: Option<&'a str>,
    pub stdout_screen_id: Option<&'a str>,
    pub stderr_screen_id: Option<&'a str>,
}

impl<'a> msg::DecodeMessage<'a> for ServerHello<'a> {
    fn decode_message(msg: &'a msg::Message) -> Option<Self> {
        if msg.parsed_type() != MessageType::ServerHello {
            return None;
        }
        let (client_id, stdin_screen_id, stdout_screen_id, stderr_screen_id) =
            msg.arguments().exactly4()?;
        Some(ServerHello {
            client_id,
            stdin_screen_id,
            stdout_screen_id,
            stderr_screen_id,
        })
    }
}

impl<'a> msg::EncodeMessage for ServerHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, "server-hello", 4);
        f.add_argument(&self.client_id);
        f.add_argument(&self.stdin_screen_id);
        f.add_argument(&self.stdout_screen_id);
        f.add_argument(&self.stderr_screen_id);
        f.finalize()
    }
}
