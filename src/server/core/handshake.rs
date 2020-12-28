/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::common::core::msg::DecodeMessage;
use crate::msg::posix::{ClientHello, ServerHello, StdinHello, StdoutHello};
use crate::server;
use crate::server::HandlerError::InvalidMessage;
use crate::server::{MessageConnector, StdoutConnector};

///A [HandshakeHandler](../trait.HandshakeHandler.html) providing basic support for the client
///handshakes defined in [`vt6/foundation`](https://vt6.io/std/foundation/) and the platform
///integration modules supported by this crate (currently only
///[`vt6/posix`](https://vt6.io/std/posix/)).
#[derive(Default)]
pub struct HandshakeHandler<Next>(Next);

impl<A: server::Application, Next: server::HandshakeHandler<A>> server::HandshakeHandler<A>
    for HandshakeHandler<Next>
{
}

impl<A: server::Application, Next: server::HandshakeHandler<A>> server::Handler<A>
    for HandshakeHandler<Next>
{
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    ) -> Result<(), server::HandlerError> {
        let d = conn.dispatch();
        let app = d.application();

        match msg.parsed_type().as_str() {
            "posix1.stdin-hello" => {
                let msg = StdinHello::decode_message(msg).ok_or(InvalidMessage)?;
                let identity = app.authorize_stdin(msg.secret).ok_or(InvalidMessage)?;
                conn.set_state(server::ConnectionState::Stdin(identity));
                Ok(())
            }
            "posix1.stdout-hello" => {
                let msg = StdoutHello::decode_message(msg).ok_or(InvalidMessage)?;
                let identity = app.authorize_stdout(msg.secret).ok_or(InvalidMessage)?;
                let connector = A::StdoutConnector::new(identity);
                conn.set_state(server::ConnectionState::Stdout(connector));
                Ok(())
            }
            "posix1.client-hello" => {
                let msg = ClientHello::decode_message(msg).ok_or(InvalidMessage)?;
                let identity = app.authorize_client(msg.secret).ok_or(InvalidMessage)?;
                let connector = A::MessageConnector::new(identity.clone());
                conn.set_state(server::ConnectionState::Msgio(connector));
                let reply = ServerHello {
                    client_id: identity.client_id(),
                    stdin_screen_id: identity.stdin_screen_id(),
                    stdout_screen_id: identity.stdout_screen_id(),
                    stderr_screen_id: identity.stderr_screen_id(),
                };
                conn.enqueue_message(&reply);
                Ok(())
            }
            _ => self.0.handle(msg, conn),
        }
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    ) {
        self.0.handle_error(err, conn);
    }
}
