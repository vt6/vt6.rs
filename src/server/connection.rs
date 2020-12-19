/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

///State machine for a client socket.
#[derive(Debug)]
pub enum ConnectionState<A: server::Application> {
    ///The client socket has just been opened and we're waiting for the first message from the
    ///client before choosing the actual socket type.
    Handshake,
    ///This socket is in msgio mode because of a successful client-hello message.
    Msgio(A::MessageConnector),
    ///This socket is in stdin mode because of a successful stdin-hello message.
    Stdin(server::ScreenIdentity),
    ///This socket is in stdout mode because of a successful stdout-hello message.
    Stdout(A::StdoutConnector),
    ///This socket is currently being torn down. No further IO shall be performed on the socket and
    ///all resources relating to it shall be released.
    Teardown,
}

impl<A: server::Application> ConnectionState<A> {
    ///Returns the name of the state, e.g. "Msgio" if `self` is `ConnectionState::Msgio(...)`.
    ///This function is useful for formatting error messages.
    ///
    ///```ignore
    ///format!("expected socket in state \"Msgio\", got \"{}\"", connection.state().type_name())
    ///```
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Handshake => "Handshake",
            Self::Msgio(_) => "Msgio",
            Self::Stdin(_) => "Stdin",
            Self::Stdout(_) => "Stdout",
            Self::Teardown => "Teardown",
        }
    }
}

///Generic interface for a receive buffer.
///
///The actual buffer type is tied to the concrete [Dispatch](trait.Dispatch.html) and
///instances are created and filled by it. The Dispatch then calls `handle_incoming` on the
///[Connection](struct.Connection.html) to process the contents of the receive buffer.
pub trait ReceiveBuffer {
    ///Returns a reference to the filled part of the buffer.
    fn contents(&self) -> &[u8];
    ///Discards the first `len` bytes from the buffer, so that `self.contents()` afterwards refers
    ///only to the rest, after those bytes.
    fn discard(&mut self, len: usize);
}

///A single client connection to the server socket.
pub struct Connection<A: server::Application, D: server::Dispatch<A>> {
    dispatch: D,
    id: D::ConnectionID,
    state: ConnectionState<A>,
}

impl<A: server::Application, D: server::Dispatch<A>> Connection<A, D> {
    ///Creates a new connection. This interface is usually only called by the Dispatch when
    ///accepting a client connection to the server socket.
    pub fn new(dispatch: D, id: D::ConnectionID) -> Self {
        Self {
            dispatch,
            id,
            state: ConnectionState::Handshake,
        }
    }

    ///Returns a reference to the dispatch. Handlers that only get a reference to a Connection
    ///instance can use this method to talk to the dispatch.
    pub fn dispatch(&self) -> D {
        self.dispatch.clone()
    }

    ///Returns the internal ID of this connection. The ID is unique within the Dispatch instance
    ///that manages this connection.
    pub fn id(&self) -> D::ConnectionID {
        self.id.clone()
    }

    ///Returns the current state of this connection.
    pub fn state(&self) -> &ConnectionState<A> {
        &self.state
    }

    ///Switch this connection into a different state. Handshake handlers can use this method to set
    ///the socket from handshake mode into msgio, stdin or stdout mode. Also, any handler wishing
    ///to dismantle the connection (e.g. because of a fatal error) can use this method to set the
    ///socket in teardown mode, which will cause the dispatch to shut down the connection.
    pub fn set_state(&mut self, state: ConnectionState<A>) {
        self.state = state;
    }

    ///A shorthand for extracting the MessageConnector out of `self.state()`. Returns `None` when
    ///not in msgio mode.
    pub fn message_connector(&mut self) -> Option<&mut A::MessageConnector> {
        use ConnectionState::*;
        match self.state {
            Msgio(ref mut c) => Some(c),
            _ => None,
        }
    }

    ///A shorthand for extracting the StdoutConnector out of `self.state()`. Returns `None` when
    ///not in stdout mode.
    pub fn stdout_connector(&mut self) -> Option<&mut A::StdoutConnector> {
        use ConnectionState::*;
        match self.state {
            Stdout(ref mut c) => Some(c),
            _ => None,
        }
    }

    ///A shorthand for `self.dispatch().enqueue_message(self, action)`. See
    ///[over here](trait.Dispatch.html#tymethod.enqueue_message) for details.
    pub fn enqueue_message<M: msg::EncodeMessage>(&mut self, msg: &M) {
        self.dispatch().enqueue_message(self, msg)
    }

    ///Handle data sent by the client. This interface is called by the Dispatch whenever data has
    ///been read from the client socket associated with this Connection instance.
    pub fn handle_incoming<B: ReceiveBuffer>(&mut self, buf: &mut B) {
        if !buf.contents().is_empty() {
            use ConnectionState::*;
            match self.state {
                Handshake => self.handle_incoming_msgio::<B, A::HandshakeHandler>(buf),
                Msgio(_) => self.handle_incoming_msgio::<B, A::MessageHandler>(buf),
                Stdin(_) => unimplemented!(),
                Stdout(_) => unimplemented!(),
                Teardown => {}
            }
        }
    }

    fn handle_incoming_msgio<B: ReceiveBuffer, H: server::Handler<A> + Default>(
        &mut self,
        buf: &mut B,
    ) {
        let handler = H::default();
        match msg::Message::parse(buf.contents()) {
            Ok((msg, bytes_parsed)) => {
                handler.handle(&msg, self);
                buf.discard(bytes_parsed);
            }
            Err(e) if e.kind == msg::ParseErrorKind::UnexpectedEOF => {
                //if we don't have a full message yet, wait until the next read
                return;
            }
            Err(e) => {
                handler.handle_error(&e, self);
                //After a parse error, recover by skipping ahead to the next possible start of
                //a message, i.e. the next `{` sign. [vt6/foundation, sect. 3.3]
                //
                //The .skip(1) ensures that we don't skip by 0 bytes.
                let bytes_to_discard = match buf.contents().iter().skip(1).position(|&b| b == b'{')
                {
                    Some(offset) => offset + 1,   //`+1` compensates the effect of .skip(1)
                    None => buf.contents().len(), //no `{` at all -> everything is garbage
                };
                let n = server::Notification::IncomingBytesDiscarded(
                    &buf.contents()[0..bytes_to_discard],
                );
                self.dispatch.application().notify(&n);
                buf.discard(bytes_to_discard);
            }
        }
        //handling the previous message (or error) may have changed into a different state, so
        //tail-call back into handle_incoming() to disambiguate again
        self.handle_incoming(buf)
    }
}
