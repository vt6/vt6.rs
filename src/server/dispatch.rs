/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

///A reference to the IO job or worker thread managing the server socket.
///
///The implementation of this type encapsulates the handling of the server socket and of client
///connections to it, and thus depends on which IO library or framework you use. This crate
///includes an implementation for use with the Tokio library in the
///[vt6::server::tokio](tokio/index.html) submodule if the "use_tokio" feature is enabled on the
///crate. Similar implementations for other IO libraries may be added in the future, but you can
///always provide your own if the ones supplied with this crate don't fit your use case.
pub trait Dispatch<A: server::Application>: Clone + Sized {
    ///The dispatch assigns a unique ID of this type to every [Connection](struct.Connection.html)
    ///managed by it.
    type ConnectionID: Clone + Send + Sync;

    ///A reference to the application core.
    fn application(&self) -> &A;

    ///Registers a broadcast action.
    ///
    ///When handling input or requests sent by a client, the respective handler only has a
    ///`&mut Connection` for the connection where the input was received, but some messages require
    ///sending responses to other connections, too (e.g. when a `core1.set` message sets a property
    ///value, all other connections with a subscription on the same property may need to be
    ///notified). But the dispatch cannot have a method like
    ///
    ///```ignore
    ///fn connections_mut(&self) -> impl Iterator<Item=&mut server::Connection<A, Self>>;
    ///```
    ///
    ///for the handler to call, since the handler already has one of the `&mut Connection` in
    ///question and we cannot get a second mutable reference to it. What the handler actually does
    ///is to enqueue a broadcast action. The dispatch takes ownership of the action and executes it
    ///as soon as all `&mut Connection` references have been returned to it.
    fn enqueue_broadcast(
        &self,
        action: Box<dyn Fn(&mut server::Connection<A, Self>) + Send + Sync>,
    );

    ///Writes a message into the send buffer of the given connection.
    ///
    ///Calls are only allowed when `conn.state()` is `Handshake` or `Msgio`. If this condition is
    ///not met, the implementation may choose to ignore the message or to panic.
    ///
    ///You need a `&mut Connection` reference to call this, so this method can easily be called
    ///inside [handlers](trait.Handler.html). If you want to send messages while not handling a
    ///client message, you need to `enqueue_broadcast()` your action and have the dispatch get back
    ///to you when it's ready to give you a `&mut Connection`.
    fn enqueue_message<M: msg::EncodeMessage>(
        &self,
        conn: &mut server::Connection<A, Self>,
        msg: &M,
    );

    ///Writes standard input into the send buffer of the given connection.
    ///
    ///Calls are only alowed when `conn.state()` is `Stdin`. If this condition is not met, the
    ///implementation may choose to ignore the message or to panic.
    ///
    ///You need a `&mut Connection` reference to call this, so you probably need to
    ///`enqueue_broadcast()` your request and have the dispatch get back to you when it's ready to
    ///give you a `&mut Connection`.
    ///
    ///# Examples
    ///
    ///To send input for the screen with the ID "example" to the respective client's stdin:
    ///
    ///```ignore
    ///let buf: Vec<u8> = "hello stdin".into();
    ///let screen = vt6::server::ScreenIdentity::new("example");
    ///dispatch.enqueue_broadcast(Box::new(move |conn| {
    ///    if conn.state().can_receive_stdin_for_screen(&screen) {
    ///        conn.enqueue_stdin(&buf);
    ///    }
    ///}));
    ///```
    fn enqueue_stdin(&self, conn: &mut server::Connection<A, Self>, buf: &[u8]);
}
