/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg::DecodeMessage;
use crate::common::core::{msg, ModuleIdentifier};
use crate::msg::{Have, Want};
use crate::server;

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

///A [MessageHandler](../trait.MessageHandler.html) providing basic support for the client
///handshakes defined in [vt6::foundation](https://vt6.io/std/foundation/) and the platform
///integration modules supported by this crate (currently only
///[vt6::posix](https://vt6.io/std/posix/)).
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

        //TODO handle core1.sub and core1.set (deferred until we have an actual property)

        //TODO handle core1.lifetime-end

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
