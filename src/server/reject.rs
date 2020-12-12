/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

///A [MessageHandler](../trait.MessageHandler.html) that provides the default behavior for unknown
///or invalid client messages:
///
///* The response for syntactically invalid messages and unexpected eternal messages is a
///  ["nope" message](https://vt6.io/std/foundation/#section-5-2).
///* The response for all other syntactically valid messages is a negative
///  ["have" message](https://vt6.io/std/foundation/#section-4-2),
///  indicating that the respective module is apparently not supported by this server.
///
///This handler is usually the last in every MessageHandler chain. Valid messages will be
///processeed by an earlier handler and never reach this handler.
#[derive(Default)]
pub struct MessageHandler;

impl<A: server::Application> server::MessageHandler<A> for MessageHandler {}

impl<A: server::Application> server::Handler<A> for MessageHandler {
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    ) {
        use crate::common::core::MessageType::*;
        if let Scoped(mt) = msg.parsed_type() {
            conn.enqueue_message(|buf| {
                let mut f = msg::MessageFormatter::new(buf, "have", 1);
                f.add_argument(&mt.module());
                f.finalize()
            })
        } else {
            conn.enqueue_message(|buf| msg::MessageFormatter::new(buf, "nope", 0).finalize())
        }
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        _err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    ) {
        conn.enqueue_message(|buf| msg::MessageFormatter::new(buf, "nope", 0).finalize())
    }
}

///A [HandshakeHandler](../trait.HandshakeHandler.html) that reacts to anything by closing the
///connection.
///
///This handler is usually the last in every HandshakeHandler chain. Valid handshake messages will
///be processed by an earlier handler and never reach this handler. Therefore, if any message
///reaches this handler, the handshake must have failed, in which case closing the connection is
///the appropriate response.
#[derive(Default)]
pub struct HandshakeHandler;

impl<A: server::Application> server::HandshakeHandler<A> for HandshakeHandler {}

impl<A: server::Application> server::Handler<A> for HandshakeHandler {
    fn handle<D: server::Dispatch<A>>(
        &self,
        _msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    ) {
        conn.set_state(server::ConnectionState::Teardown);
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        _err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    ) {
        conn.set_state(server::ConnectionState::Teardown);
    }
}
