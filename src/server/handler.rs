/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

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
///use vt6::server::{core, reject, sig, term};
///impl Application for MyApplication {
///    type MessageHandler = MyLoggingHandler<core::Handler<sig::Handler<term::Handler<MyErrorHandler<reject::Handler>>>>>;
///    //... other fields elided ...
///}
///```
///
///Handlers are supposed to be stateless. In fact, a new instance of the handler will be created
///for every message, hence the `Default` bound on this trait. Handler implementations will usually
///only contain the next handler instance (if any) and thus have a final `std::mem::size_of()` of 0.
pub trait Handler<A: server::Application>: Default {
    ///Handle a message sent by the client on the given connection.
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    );

    ///Handle a syntactically incorrect message or other unintelligible input sent by the client on
    ///the given connection. Most handlers will just forward to the next handler in line. This
    ///method is only interesting for handlers that do logging and error handling. If you don't end
    ///your handler chain with your own custom error handler, make sure to use one of the handlers
    ///in the [vt6::server::reject](reject/index.html) module to handle errors as the spec demands.
    fn handle_error<D: server::Dispatch<A>>(
        &self,
        err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    );
}

///Marker trait for [handlers](trait.Handler.html) that can be used on msgio sockets.
pub trait MessageHandler<A: server::Application>: Handler<A> {}

///Marker trait for [handlers](trait.Handler.html) that can be used during the client handshake
///phase.
pub trait HandshakeHandler<A: server::Application>: Handler<A> {}
