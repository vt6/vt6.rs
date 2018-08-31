/******************************************************************************
*
*  Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
*
******************************************************************************/

use core::{msg, EncodeArgument};
use server::Connection;

///The error type returned by [`Handler::handle()`](trait.Handler.html) and
///[`EarlyHandler::handle()`](trait.EarlyHandler.html).
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum HandlerError {
    ///The message given to `handle()` was invalid. The caller may need to
    ///answer with an error notification, e.g. a `nope` message.
    InvalidMessage,
    ///The handler tried to send a response message, but the connection's send
    ///buffer was not large enough. The caller should call `handle()` again once
    ///the send buffer is large enough. The contained `usize` value indicates
    ///how many bytes could not be written into the target buffer.
    SendBufferTooSmall(usize),
}

///Evaluate the given callback and convert a `None` result into
///[`HandlerError::InvalidMessage`](enum.HandlerError.html). This function is
///intended for use within [`Handler::handle()`](trait.Handler.html)
///implementations. When parsing message arguments, parsing code that returns
///`Result` or `Option` types can be wrapped in this function to enable usage of
///the `?` operator.
///
///TODO: This is a provisional API. When the `try_trait` language feature
///gets stable, replace this with a `std::convert::From<std::option::NoneError>`
///implementation on HandlerError.
///Tracking issue: <https://github.com/rust-lang/rust/issues/42327>
pub fn try_or_message_invalid<T, F: FnOnce() -> Option<T>>(action: F) ->
Result<T, HandlerError> {
    action().ok_or(HandlerError::InvalidMessage)
}

///A handler is the part of a VT6 server that processes VT6 messages. This trait
///is the correct one for most handlers, but early handlers that come before the
///[vt6::core::server::Handler](../core/server/struct.Handler.html) must
///implement [EarlyHandler](trait.EarlyHandler.html) instead.
///
///# Composition of handlers
///
///Handlers are expected to be composed in a middleware-like fashion. An outer
///handler can then pass any messages or other requests not known to it to the
///next handler in the chain.
///
///```rust,ignore
///use vt6;
///
///let handler = vt6::server::RejectHandler {};
///let handler = FirstCustomHandler::new(handler);
///let handler = SecondCustomHandler::new(handler);
///let handler = vt6::core::server::Handler::new(handler);
///```
///
///As shown above, the innermost handler factory is usually going to be
///[vt6::server::RejectHandler](struct.RejectHandler.html), which rejects any
///messages that have not already been recognized by other handlers along the
///way.
///
///The outermost handler factory is usually going to be
///[vt6::core::server::Handler](../core/server/struct.Handler.html), which
///translates module negotiation and property pub/sub messages into more
///specific requests for the other handlers.
///
///# How to handle state
///
///Handlers are usually application-global: Single instances of them are used
///for all server connections. Handlers should therefore store all state pertaining to
///individual connections in the [Connection object](trait.Connection.html)
///passed into each of their methods.
///
///Handlers should always be `Sync` because a server may be servicing different
///connections from different threads. When handlers use interior mutability to
///store global state pertaining to all server connections (e.g.
///application-global statistics), they should use thread-safe containers such
///as std::sync::Mutex or std::sync::RwLock.
///
///# Implementing handlers
///
///Since handlers cannot hold any connection-scoped state, they must cooperate
///with the connection object passed into each of their methods. To this end,
///handlers are supposed to be parametrized over a type of connection `C` in the
///same way as this trait. The handler can then add its own trait bound to `C`,
///requiring the connection to implement a custom trait that provides the
///necessary behavior.
///
///```rust,ignore
///use std::marker::PhantomData;
///use vt6::core::msg;
///use vt6::server::{Connection, Handler};
///
///trait ExampleConnection: Connection {
///    fn frobnicate(&mut self);
///}
///
///struct ExampleHandler<C: Connection + ExampleConnection, H: Handler<C>> {
///    next: H,
///    phantom: PhantomData<C>,
///}
///
///impl<C: Connection + ExampleConnection> Handler<C> for ExampleHandler<C> {
///    fn handle(&self, msg: &msg::Message, conn: &mut C) -> bool {
///        if msg.type_name() == ("example", "frobnicate") {
///            //... argument validation elided for brevity ...
///            conn.frobnicate();
///        } else {
///            self.next.handle(msg, conn)
///        }
///    }
///
///    //... other method implementations elided for brevity ...
///}
///```
///
///The basic idea is that handlers decode VT6 messages into method calls on the
///connection object.
///
///Note that, when using `PhantomData<C>` as in the example above, the auto trait implementations
///of Send and Sync have an unhelpful "where C: Send/Sync" bound, so it is recommended to provide
///your own trait implementations without that bound:
///
///```rust,ignore
///unsafe impl<C: Connection + ExampleConnection, H: Handler<C>> Send for ExampleHandler<C, H> where H: Send {}
///unsafe impl<C: Connection + ExampleConnection, H: Handler<C>> Sync for ExampleHandler<C, H> where H: Sync {}
///```
pub trait Handler<C: Connection> {
    ///This method is called for each message from the client that is received
    ///on this handler's server connection, except for messages that the
    ///[vt6::core::server::Handler](../core/server/struct.Handler.html) parses
    ///and forwards to the following handlers using the more specific methods
    ///below.
    ///
    ///The `send_buffer` argument is the free part of the send buffer. The handler can use the
    ///[MessageFormatter](../core/msg/struct.MessageFormatter.html) to append messages to the
    ///buffer.
    ///
    ///The return value shall indicate whether the received message was valid.
    ///If the handler does not know how to handle this message type, it may
    ///recurse into the next handler if there is one.
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Result<usize, HandlerError>;

    ///This method is called for each `want` message that requests usage of a
    ///module. If the `want` message offers multiple major versions, this
    ///function is called once for each offered major version.
    ///
    ///If the requested module depends on other modules being used on this
    ///connection, the implementation shall use `conn.is_module_enabled()` to
    ///check these dependencies.
    ///
    ///If the handler can agree to using this module, it shall return the minor
    ///version supported for this module and major version. Otherwise, it may
    ///recurse into the next handler if there is one.
    fn can_use_module(&self, name: &str, major_version: u16, conn: &C) -> Option<u16>;

    ///This method is called once for each argument of a `core.sub` message, or
    ///each pair of arguments of a `core.set` message. If the property does not
    ///exist, the handler shall return `None`. Otherwise, it shall
    ///
    ///1. attempt to set the property's value to the requested value *if*
    ///   `requested_value.is_some()`,
    ///
    ///2. return the property's current (or new) value (which may be different
    ///   from the requested one) wrapped in `Some`,
    ///
    ///3. record a subscription to this property in `conn`. This means that,
    ///   whenever the property changes after this call, the handler shall send
    ///   a `core.pub` message using the `sender` that was supplied to the
    ///   handler's factory when instantiating this handler.
    ///
    ///If the handler does not know this property, it may recurse into the
    ///next handler if there is one.
    fn get_set_property<'c>(&self, name: &str, requested_value: Option<&[u8]>, conn: &'c mut C) -> Option<&'c EncodeArgument>;
}

///A handler is the part of a VT6 server that processes VT6 messages. This trait
///is only used for early handlers that come before the
///[vt6::core::server::Handler](../core/server/struct.Handler.html). Most
///handlers will want to implement the regular [Handler
///trait](trait.Handler.html).
///
///Refer to the documentation on the [Handler trait](trait.Handler.html) for
///more details about the concept of handlers.
pub trait EarlyHandler<C: Connection> {
    ///This method is called for each message from the client that is received
    ///on this handler's server connection, unless a previous handler
    ///transformed the message into something else.
    ///
    ///The `send_buffer` argument is the free part of the send buffer. The handler can use the
    ///[MessageFormatter](../core/msg/struct.MessageFormatter.html) to append messages to the
    ///buffer.
    ///
    ///The return value shall indicate whether the received message was valid.
    ///If the handler does not know how to handle this message type, it may
    ///recurse into the next handler if there is one.
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Result<usize, HandlerError>;
}
