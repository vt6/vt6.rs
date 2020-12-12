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
    WaitingForClientHello,
    ///This socket is in msgio mode because of a successful client-hello message.
    Msgio(A::MessageConnector),
    ///This socket is in stdin mode because of a successful stdin-hello message.
    Stdin(A::StdinConnector),
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
            Self::WaitingForClientHello => "WaitingForClientHello",
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
    pub fn new(dispatch: D, id: D::ConnectionID) -> Self {
        Self {
            dispatch,
            id,
            state: ConnectionState::WaitingForClientHello,
        }
    }

    pub fn dispatch(&self) -> D {
        self.dispatch.clone()
    }

    pub fn id(&self) -> D::ConnectionID {
        self.id.clone()
    }

    pub fn state(&self) -> &ConnectionState<A> {
        &self.state
    }

    pub fn set_state(&mut self, state: ConnectionState<A>) {
        self.state = state;
    }

    pub fn msgio_connector(&mut self) -> Option<&mut A::MessageConnector> {
        use ConnectionState::*;
        match self.state {
            Msgio(ref mut c) => Some(c),
            _ => None,
        }
    }

    pub fn handle_incoming<B: ReceiveBuffer>(&mut self, buf: &mut B) {
        if !buf.contents().is_empty() {
            use ConnectionState::*;
            match self.state {
                WaitingForClientHello => self.handle_incoming_msgio::<B, A::HandshakeHandler>(buf),
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
                self.dispatch
                    .notify(&server::Notification::IncomingBytesDiscarded(
                        &buf.contents()[0..bytes_to_discard],
                    ));
                buf.discard(bytes_to_discard);
            }
        }
        //handling the previous message (or error) may have changed into a different state, so
        //tail-call back into handle_incoming() to disambiguate again
        self.handle_incoming(buf)
    }
}
