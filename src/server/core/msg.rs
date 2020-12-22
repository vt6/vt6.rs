/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg::DecodeMessage;
use crate::common::core::{msg, ModuleIdentifier, OwnedClientID};
use crate::msg::core::*;
use crate::msg::{Have, Nope, Want};
use crate::server;
use crate::server::{ClientIdentity, ClientSelector, ConnectionState, MessageConnector};

///Extension trait for [message handlers](../trait.MessageHandler.html).
///
///In a handler chain, handlers that occur after
///[vt6::server::core::MessageHandler](struct.MessageHandler.html) must implement this trait. The
///vt6/core message handler will decode certain types of messages and call methods from this trait
///on its inner handler to handle them.
///
///Just like for the methods on the Handler trait, implementors are supposed to defer to the next
///handler in the chain when they cannot give a definitive answer. The last handler in a chain will
///usually deny any requests not answered earlier.
pub trait MessageHandlerExt<A: server::Application>: server::MessageHandler<A> {
    ///Returns whether the given module is supported by this handler, and if so, which version is
    ///supported. This is used to answer `want` messages. For example, the message `(want foo2)`
    ///will be translated into `get_supported_module_version(m)` where `m.as_str() == "foo2"`. If
    ///the result is `Some(4)`, the reply `(have foo2.4)` will be sent. `None` indicates that the
    ///module in question is not supported at all.
    fn get_supported_module_version(&self, module: &ModuleIdentifier<'_>) -> Option<u16>;
}

///A [MessageHandler](../trait.MessageHandler.html) covering all messages defined in
///[`vt6/foundation`](https://vt6.io/std/foundation/) and [`vt6/core`](https://vt6.io/std/core/).
///
///Because this handler decodes certain messages defined in `vt6/core` and `vt6/foundation`, this
///handler requires handlers chained after it to implement
///[trait MessageHandlerExt](trait.MessageHandlerExt.html) from this module.
#[derive(Default)]
pub struct MessageHandler<Next>(Next);

impl<A: server::Application, Next: server::core::MessageHandlerExt<A>> server::MessageHandler<A>
    for MessageHandler<Next>
{
}

impl<A: server::Application, Next: server::core::MessageHandlerExt<A>> server::Handler<A>
    for MessageHandler<Next>
{
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    ) {
        //answer `want` messages: we support `core1.0` and all modules that the
        //following handlers support
        if let Some(Want(module_id)) = Want::decode_message(msg) {
            let result = if module_id.as_str() == "core1" {
                Some(0)
            } else {
                self.0.get_supported_module_version(&module_id)
            };
            let reply = match result {
                Some(v) => Have::ThisModule(module_id.with_minor_version(v)),
                None => Have::NotThisModule(module_id),
            };
            conn.enqueue_message(&reply);
            return;
        }

        //answer `core1.client-make` messages
        if let Some(msg) = ClientMake::decode_message(msg) {
            let connector = conn.message_connector().unwrap();

            //new client ID must be below this client's ID
            let selector = ClientSelector::StrictlyBelow(connector.identity().client_id());
            if !selector.contains(msg.client_id) {
                conn.enqueue_message(&Nope);
                return;
            }
            //client ID must not be in use yet
            let d = conn.dispatch();
            let selector = ClientSelector::AtOrBelow(msg.client_id);
            if d.application().has_clients(selector) {
                conn.enqueue_message(&Nope);
                return;
            }

            //convert ClientMake msg into server::ClientIdentity
            let mut id = ClientIdentity::new(&msg.client_id);
            if let Some(sid) = msg.stdin_screen_id {
                id = id.with_stdin(sid);
            }
            if let Some(sid) = msg.stdout_screen_id {
                id = id.with_stdout(sid);
            }
            if let Some(sid) = msg.stderr_screen_id {
                id = id.with_stderr(sid);
            }

            //register client and send secret to registrar
            let creds = d.application().register_client(id);
            let reply = ClientNew {
                secret: creds.secret(),
            };
            conn.enqueue_message(&reply);
            return;
        }

        //handle `core1.lifetime-end` messages
        if let Some(msg) = LifetimeEnd::decode_message(msg) {
            let connector = conn.message_connector().unwrap();
            //client ID whose lifetime ends must be below this client's ID
            let selector = ClientSelector::StrictlyBelow(connector.identity().client_id());
            if !selector.contains(msg.client_id) {
                conn.enqueue_message(&Nope);
                return;
            }

            //tear down all client connections at or below this client ID
            let owned_client_id = OwnedClientID::from(&msg.client_id);
            let d = conn.dispatch();
            d.enqueue_broadcast(Box::new(move |conn| {
                let selector = ClientSelector::AtOrBelow(owned_client_id.as_ref());
                if let ConnectionState::Msgio(ref connector) = conn.state() {
                    if selector.contains(connector.identity().client_id()) {
                        conn.set_state(ConnectionState::Teardown);
                    }
                }
            }));
            return;
        }

        //TODO handle core1.sub and core1.set (deferred until we have an actual property)

        //if we did not return yet, we did not know how to handle this message
        self.0.handle(msg, conn);
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    ) {
        self.0.handle_error(err, conn);
    }
}
