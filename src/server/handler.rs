/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, ModuleIdentifier};
use crate::server;

///Error type for `handle()` method in [trait Handler](trait.Handler.html).
///
///The value is used to trigger the baseline error handling behavior.
///[\[vt6/foundation, sect. 3.3.2\]](https://vt6.io/std/foundation/#section-3-3-2)
pub enum HandlerError {
    ///The message was of an unknown type. The caller must render a `have` response to describe
    ///support for the respective module and major version.
    UnknownMessageType,
    ///The message type was recognized, but the message was semantically invalid. The caller must
    ///render a `nope` response.
    InvalidMessage,
}

///The main trait for message handlers.
///
///Handlers are used to parse and handle messages sent by the client on fresh sockets
///([HandshakeHandler](trait.HandshakeHandler.html)) and on msgio sockets
///([MessageHandler](trait.MessageHandler.html)).
///
///This crate includes various modular handler types, most of them implementing support for one
///specific VT6 module. Handlers can be composed by chaining, similar to the "middlewares" that
///exist in most HTTP server frameworks. Custom handler types can be mixed and matched with those
///in the library if non-standard VT6 modules need to be supported, or for application-specific
///high-level concerns like logging and error handling. A typical handler chain might look like
///this:
///
///```ignore
///use vt6::server::{core, sig, term, RejectHandler};
///impl Application for MyApplication {
///    type MessageHandler = MyLoggingHandler<core::MessageHandler<sig::MessageHandler<term::MessageHandler<MyErrorHandler<RejectHandler>>>>>;
///    //... other fields elided ...
///}
///```
///
///Handlers are supposed to be stateless. In fact, a new instance of the handler will be created
///for every message, hence the `Default` bound on this trait. Handler implementations will usually
///only contain the next handler instance (if any) and thus have a final `std::mem::size_of()` of 0.
pub trait Handler<A: server::Application>: Default {
    ///Handle a message sent by the client on the given connection.
    ///
    ///Handlers are **not** responsible for the baseline error behavior defined in
    ///[\[vt6/foundation, sect. 3.3.2\]](https://vt6.io/std/foundation/#section-3-3-2). This
    ///behavior is implemented internally in this crate, depending on which HandlerResult is
    ///returned from handle().
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    ) -> Result<(), HandlerError>;

    ///Handle a syntactically incorrect message or other unintelligible input sent by the client on
    ///the given connection. Most handlers will just forward to the next handler in line. This
    ///method is only interesting for handlers that do logging and error handling.
    fn handle_error<D: server::Dispatch<A>>(
        &self,
        err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    );
}

///Marker trait for [handlers](trait.Handler.html) that can be used on msgio sockets.
pub trait MessageHandler<A: server::Application>: Handler<A> {
    ///Returns whether the given module is supported by this handler, and if so, which version is
    ///supported. This is used to answer `want` messages, and also to render error responses. For
    ///example, the message `(want foo2)` will be translated into `get_supported_module_version(m)`
    ///where `m.as_str() == "foo2"`. If the result is `Some(4)`, the reply `(have foo2.4)` will be
    ///sent. `None` indicates that the module in question is not supported at all, in which case
    ///`(have foo2)` would be sent.
    fn get_supported_module_version(&self, module: &ModuleIdentifier<'_>) -> Option<u16>;
}

///Marker trait for [handlers](trait.Handler.html) that can be used during the client handshake
///phase.
pub trait HandshakeHandler<A: server::Application>: Handler<A> {}
