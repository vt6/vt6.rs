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

use common::core::msg;
use server::Connection;

///A handler is the part of a VT6 server that processes VT6 messages. This trait
///is the correct one for most handlers, but early handlers that come before the
///[vt6::server::core::Handler](core/struct.Handler.html) must
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
///let handler = vt6::server::core::Handler::new(handler);
///```
///
///As shown above, the innermost handler factory is usually going to be
///[vt6::server::RejectHandler](struct.RejectHandler.html), which rejects any
///messages that have not already been recognized by other handlers along the
///way.
///
///The outermost handler factory is usually going to be
///[vt6::server::core::Handler](core/struct.Handler.html), which
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
///use vt6::common::core::msg;
///use vt6::server::{Connection, Handler};
///
///trait ExampleConnection: Connection {
///    fn frobnicate(&mut self);
///}
///
///struct ExampleHandler<H> {
///    next: H,
///}
///
///impl<C, H> Handler<C> for ExampleHandler<H>
///     where H: Handler<C>,
///           C: Connection + ExampleConnection
///{
///    fn handle(&self, msg: &msg::Message, conn: &mut C) -> Option<usize> {
///        if msg.type_name() == ("example", "frobnicate") {
///            //... argument validation elided for brevity ...
///            conn.frobnicate();
///            return Some(0);
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
pub trait Handler<C: Connection> {
    ///This method is called for each message from the client that is received
    ///on this handler's server connection, unless a previous handler
    ///transformed the message into something else.
    ///
    ///The `send_buffer` argument is the free part of the send buffer. The
    ///handler can use the
    ///[MessageFormatter](../common/core/msg/struct.MessageFormatter.html) to
    ///append messages to the buffer. The caller must ensure that
    ///`send_buffer.len() <= conn.max_server_message_length()`, in other words:
    ///The send buffer must be large enough to hold at least one message
    ///completely.
    ///
    ///The return value shall be `None` if `message` was invalid, or
    ///`Some(bytes_written)` to indicate how many bytes were written into the
    ///send buffer.
    ///
    ///If the handler does not know how to handle this message type, it may
    ///recurse into the next handler if there is one.
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>;

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

    ///This method is called once for each `core.sub` message. The
    ///implementation shall check if `name` refers to a property in a module
    ///that the server has agreed to on this connection. If so, it shall
    ///
    ///1. report the property's current (or new) value by calling
    ///   [`MessageFormatter::publish_property(send_buffer, name,
    ///   value)`](../common/core/msg/struct.MessageFormatter.html),
    ///
    ///2. record a subscription to this property in `conn`. This means that,
    ///   whenever the property changes after this call, the handler shall send
    ///   a `core.pub` message using the `sender` that was supplied to the
    ///   handler's factory when instantiating this handler. This step can be
    ///   omitted for read-only properties.
    ///
    ///The return value shall be either `None` (if `name` is not a valid
    ///property or the required modules were not yet agreed to), or the return
    ///value from `MessageFormatter::publish_property()`.
    fn handle_sub(&self, name: &str, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>;

    ///This method is called once for each `core.set` message. The
    ///implementation shall check if `name` refers to a property in a module
    ///that the server has agreed to on this connection. If so, it shall
    ///
    ///1. attempt to set the property's value to `requested_value`
    ///
    ///2. report the property's current (or new) value (which may be different
    ///   from the requested one) by calling
    ///   [`MessageFormatter::publish_property(send_buffer, name,
    ///   value)`](../common/core/msg/struct.MessageFormatter.html),
    ///
    ///The return value shall be either `None` (if `name` is not a valid
    ///property or the required modules were not yet agreed to), or the return
    ///value from `MessageFormatter::publish_property()`.
    fn handle_set(&self, name: &str, requested_value: &[u8], conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>;
}

///A handler is the part of a VT6 server that processes VT6 messages. This trait
///is only used for early handlers that come before the
///[vt6::server::core::Handler](core/struct.Handler.html). Most handlers will
///want to implement the regular [Handler trait](trait.Handler.html).
///
///Refer to the documentation on the [Handler trait](trait.Handler.html) for
///more details about the concept of handlers.
pub trait EarlyHandler<C: Connection> {
    ///See documentation on `Handler::handle()`. This method fulfils the same
    ///contract, except that `core.set` and `core.sub` messages may be passed to
    ///it.
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>;
}
