/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::common::core::msg::DecodeMessage;
use crate::msg::{Have, Nope, Want};
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
        //answer everything with a negative `have` if possible, or with a `nope` if not
        use crate::common::core::MessageType;
        match msg.parsed_type() {
            MessageType::Scoped(mt) => conn.enqueue_message(&Have::NotThisModule(mt.module())),
            MessageType::Want => match Want::decode_message(msg) {
                Some(Want(m)) => conn.enqueue_message(&Have::NotThisModule(m)),
                None => conn.enqueue_message(&Nope),
            },
            _ => conn.enqueue_message(&Nope),
        }
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        _err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    ) {
        conn.enqueue_message(&crate::msg::Nope)
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
