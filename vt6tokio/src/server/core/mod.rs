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

mod bidi_byte_stream;
mod server;
pub use self::server::*;

use futures::sync::mpsc;
use vt6::server as vt6s;

///Extends the `vt6::server::Connection` trait with integration points for
///[`struct Server`](struct.Server.html).
pub trait Connection: vt6s::core::Connection {
    ///A reference to the application's shared state that all connections work
    ///on, including e.g. the terminal document. This type is typically an
    ///`Arc<Mutex<...>>`.
    ///
    ///If your application does not need this, set this type to `()`.
    type ModelRef: Clone + Send;
    ///An application-specific type, usually an enum, that can be generated by
    ///this connection when handling incoming messages in order to wake up other
    ///parts of the application (outside the Server future). A typical usage is
    ///to send an outgoing event to wake up the GUI thread when the model has
    ///changed and the GUI needs to be re-rendered.
    ///
    ///If your application does not need this, set this type to `()` and just
    ///throw away the `mpsc::Receiver` for these events after creating the
    ///server future. The server will never send outgoing events itself since it
    ///does not know the concrete `OutgoingEvent` type.
    type OutgoingEvent;

    ///Construct a new connection object. This is called by the server future
    ///when a new connection is accepted on the server socket. The arguments
    ///`model` and `event_tx` are clones of the same arguments originally passed
    ///to [`Server::new()`](struct.Server.html). The `id` uniquely identifies
    ///this connection without any intrinsic meaning. It should only be used for
    ///log messages.
    fn new(id: u32, model: Self::ModelRef, event_tx: mpsc::Sender<Self::OutgoingEvent>) -> Self;

    ///Return the connection ID that was given to `new()`. This function is used
    ///frequently for generating log messages etc. and should be cheap.
    fn id(&self) -> u32;

    ///This is called by the server future whenever this connection is in stdio
    ///mode and we receive text from the client. Per the `vt6/term`
    ///specification, the implementation must assume `bytes_received` to be
    ///encoded in UTF-8 and perform a lossy decoding, e.g. with
    ///`String::from_utf8_lossy()`.
    ///
    ///TODO This feels like it belongs in vt6::server::Connection or maybe
    ///vt6::server::term::Connection (once that exists). That's also why I don't
    ///have the caller do the decoding; lossy decoding requires allocations.
    fn handle_standard_output(&mut self, bytes_received: &[u8]);
}

///Events that can be sent from outside a server future to cause the server
///future to do work.
///
///TODO Once the vt6::server::Handler trait gains its own notion of events (for
///subscription updates etc.), maybe merge this into there?
pub enum IncomingEvent {
    ///Indicates the availability of user input. The contained string will
    ///immediately be sent to the most recently opened standard input.
    UserInput(String),
}
