/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, ClientID};

const CLIENT_HELLO: &str = "posix1.client-hello";
const PARENT_HELLO: &str = "posix1.parent-hello";
const SERVER_HELLO: &str = "posix1.server-hello";
const STDIN_HELLO: &str = "posix1.stdin-hello";
const STDOUT_HELLO: &str = "posix1.stdout-hello";

///A `posix1.client-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
#[derive(Clone, Debug)]
pub struct ClientHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for ClientHello<'a> {
    fn decode_message<'b>(msg: &'b msg::Message<'a>) -> Option<Self> {
        if msg.parsed_type().as_str() != CLIENT_HELLO {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(ClientHello { secret })
    }
}

impl<'a> msg::EncodeMessage for ClientHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, CLIENT_HELLO, 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}

///A `posix1.parent-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
#[derive(Clone, Debug)]
pub struct ParentHello<'a> {
    pub client_secret: &'a str,
    #[cfg(feature = "use_std")]
    pub server_socket_path: &'a std::path::Path,
    #[cfg(not(feature = "use_std"))]
    pub server_socket_path: &'a [u8],
}

impl<'a> msg::DecodeMessage<'a> for ParentHello<'a> {
    fn decode_message<'b>(msg: &'b msg::Message<'a>) -> Option<Self> {
        if msg.parsed_type().as_str() != PARENT_HELLO {
            return None;
        }
        let (client_secret, server_socket_path) = msg.arguments().exactly2()?;
        Some(ParentHello {
            client_secret,
            server_socket_path,
        })
    }
}

impl<'a> msg::EncodeMessage for ParentHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, PARENT_HELLO, 2);
        f.add_argument(self.client_secret);
        f.add_argument(self.server_socket_path);
        f.finalize()
    }
}

///A `posix1.server-hello` message.
///[\[vt6/foundation, sect. X.Y\]](https://vt6.io/std/foundation/#section-X-Y)
#[derive(Clone, Debug)]
pub struct ServerHello<'a> {
    pub client_id: ClientID<'a>,
    pub stdin_screen_id: Option<&'a str>,
    pub stdout_screen_id: Option<&'a str>,
    pub stderr_screen_id: Option<&'a str>,
}

impl<'a> msg::DecodeMessage<'a> for ServerHello<'a> {
    fn decode_message<'b>(msg: &'b msg::Message<'a>) -> Option<Self> {
        if msg.parsed_type().as_str() != SERVER_HELLO {
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
        let mut f = msg::MessageFormatter::new(buf, SERVER_HELLO, 4);
        f.add_argument(&self.client_id);
        f.add_argument(&self.stdin_screen_id);
        f.add_argument(&self.stdout_screen_id);
        f.add_argument(&self.stderr_screen_id);
        f.finalize()
    }
}

///A `posix1.stdin-hello` message.
///[\[vt6/posix1, sect. X.Y\]](https://vt6.io/std/posix1/#section-X-Y)
#[derive(Clone, Debug)]
pub struct StdinHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for StdinHello<'a> {
    fn decode_message<'b>(msg: &'b msg::Message<'a>) -> Option<Self> {
        if msg.parsed_type().as_str() != STDIN_HELLO {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(StdinHello { secret })
    }
}

impl<'a> msg::EncodeMessage for StdinHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, STDIN_HELLO, 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}

///A `posix1.stdout-hello` message.
///[\[vt6/posix1, sect. X.Y\]](https://vt6.io/std/posix1/#section-X-Y)
#[derive(Clone, Debug)]
pub struct StdoutHello<'a> {
    pub secret: &'a str,
}

impl<'a> msg::DecodeMessage<'a> for StdoutHello<'a> {
    fn decode_message<'b>(msg: &'b msg::Message<'a>) -> Option<Self> {
        if msg.parsed_type().as_str() != STDOUT_HELLO {
            return None;
        }
        let secret = msg.arguments().exactly1()?;
        Some(StdoutHello { secret })
    }
}

impl<'a> msg::EncodeMessage for StdoutHello<'a> {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, msg::BufferTooSmallError> {
        let mut f = msg::MessageFormatter::new(buf, STDOUT_HELLO, 1);
        f.add_argument(self.secret);
        f.finalize()
    }
}
